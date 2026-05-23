#!/usr/bin/env python3
"""Same-corpus benchmark: ecp vs. peer-tier Rust code-intelligence tools.

The existing ``scripts/parity/benchmark_vs_gitnexus.py`` compares against the
upstream Node.js GitNexus — useful for the rewrite-receipts narrative, but
not a peer-tier competitor. The promotion-readiness review (FU-2026-05-23-007)
asked for a same-corpus comparison against the *actual* Rust-tier tools we
get compared to:

    codescope (SurrealDB-backed) — https://github.com/onur-gokyildiz-bhi/codescope
    coraline  (SQLite-backed)

This script is a SCAFFOLD: it detects whether each competitor binary is on
``$PATH``, skips it cleanly when missing (with a one-line install hint), and
runs five canonical phases against every tool it can launch. The competitor
command tables below are best-effort guesses — they should be confirmed
against each tool's ``--help`` on first real run and corrected in-place
(plus a note in the PR that lands the corrected numbers).

Output:

    docs/benchmark-vs-competitors.md   — markdown table (gitignore-safe)
    docs/benchmark-vs-competitors.svg  — optional matplotlib bar chart

Usage:

    python scripts/benchmark/benchmark_vs_competitors.py
    python scripts/benchmark/benchmark_vs_competitors.py --corpus path/to/repo
    python scripts/benchmark/benchmark_vs_competitors.py --iterations 5 --no-plot
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from statistics import median

# ── Layout ──────────────────────────────────────────────────────────────────
# Resolve repo root from this file's location, not from cwd — the script must
# work whether invoked from repo root or from any nested working directory.
SCRIPT_PATH = Path(__file__).resolve()
WORKSPACE_ROOT = SCRIPT_PATH.parent.parent.parent

DEFAULT_ECP = WORKSPACE_ROOT / "target" / "release" / "ecp"
DEFAULT_CORPUS = WORKSPACE_ROOT / ".sample_repo"
DEFAULT_OUTPUT_MD = WORKSPACE_ROOT / "docs" / "benchmark-vs-competitors.md"
DEFAULT_OUTPUT_SVG = WORKSPACE_ROOT / "docs" / "benchmark-vs-competitors.svg"

# Per-command timeout. Cold-index can be slow on competitors that index
# language-by-language; allow a generous ceiling so we don't false-flag a
# SLOW tool as a FAIL. 30 min matches benchmark_vs_gitnexus.
CMD_TIMEOUT_S = 1800

# Canonical phases. Tools that don't expose a phase produce an ``N/A`` cell
# rather than being silently dropped — the absence is itself a signal.
PHASES = ("cold-index", "symbol-find", "callers", "file-context", "route-map", "cypher")


# ── Competitor spec ─────────────────────────────────────────────────────────
@dataclass(frozen=True)
class CompetitorSpec:
    """One competitor's binary name + per-phase command builders.

    ``commands`` maps phase → callable that takes (binary, corpus, symbol) and
    returns the argv list. ``None`` means the tool doesn't expose that phase
    (yields ``N/A`` in the result table).

    ``pre_clean`` (optional) returns an argv that wipes the tool's cache so
    the cold-index phase actually measures cold work — without it, idempotent
    indexers (like ``ecp admin index`` against an already-warm ``~/.ecp/``)
    register sub-10ms times that misrepresent first-run cost.

    Best-effort initial templates — verify and patch against each tool's
    actual ``--help`` on first real run. The dict-of-callables pattern keeps
    the symbol substitution explicit and lets one competitor support a phase
    via a totally different verb shape than another.
    """

    name: str
    binary: str  # name to look up via shutil.which
    install_hint: str
    commands: dict[str, callable]
    pre_clean: callable | None = None


def _ecp_cmds(symbol: str) -> dict[str, callable]:
    return {
        "cold-index": lambda b, c, _s: [str(b), "admin", "index", "--repo", str(c)],
        "symbol-find": lambda b, c, _s: [
            str(b), "find", symbol, "--repo", str(c), "--format", "json",
        ],
        "callers": lambda b, c, _s: [
            str(b), "impact", symbol, "--direction", "up", "--repo", str(c),
        ],
        "file-context": lambda b, c, _s: [str(b), "inspect", "--name", symbol, "--repo", str(c)],
        "route-map": lambda b, c, _s: [str(b), "routes", "--repo", str(c)],
        "cypher": lambda b, c, _s: [
            str(b),
            "cypher",
            f"MATCH (a) WHERE a.name='{symbol}' RETURN a LIMIT 5",
            "--repo",
            str(c),
        ],
    }


def _codescope_cmds(symbol: str) -> dict[str, callable]:
    # SurrealDB-backed; per upstream README it exposes `analyze`, `query`,
    # `inspect`, `impact`. Cypher unsupported (uses its own DSL). Verify
    # against `codescope --help` on first real run.
    return {
        "cold-index": lambda b, c, _s: [str(b), "analyze", "--repo", str(c)],
        "symbol-find": lambda b, c, _s: [str(b), "find", "--name", symbol, "--repo", str(c)],
        "callers": lambda b, c, _s: [str(b), "impact", symbol, "--repo", str(c)],
        "file-context": lambda b, c, _s: [str(b), "inspect", symbol, "--repo", str(c)],
        "route-map": None,  # unconfirmed — placeholder, mark N/A until verified
        "cypher": None,  # uses non-cypher query DSL
    }


def _coraline_cmds(symbol: str) -> dict[str, callable]:
    # SQLite-backed; coverage assumed similar to codescope. Confirm on first
    # real run and patch this dict in-place.
    return {
        "cold-index": lambda b, c, _s: [str(b), "index", str(c)],
        "symbol-find": lambda b, c, _s: [str(b), "find", symbol, "--in", str(c)],
        "callers": None,  # unconfirmed
        "file-context": lambda b, c, _s: [str(b), "show", symbol, "--in", str(c)],
        "route-map": None,
        "cypher": None,
    }


def build_specs(symbol: str, ecp_binary: Path) -> list[CompetitorSpec]:
    """Resolve symbol-bearing command builders for every competitor. The
    ``ecp_binary`` path is taken from CLI so an out-of-tree build can be
    pointed at; competitors are always resolved via ``shutil.which``."""
    return [
        CompetitorSpec(
            name="ecp",
            binary=str(ecp_binary),
            install_hint="cargo build -p egent-code-plexus --bin ecp --release",
            commands=_ecp_cmds(symbol),
            pre_clean=lambda b, c: [str(b), "admin", "drop", "--repo", str(c)],
        ),
        CompetitorSpec(
            name="codescope",
            binary="codescope",
            install_hint="cargo install codescope  # (verify package name with `cargo search codescope`)",
            commands=_codescope_cmds(symbol),
            pre_clean=None,  # confirm + add when codescope is wired
        ),
        CompetitorSpec(
            name="coraline",
            binary="coraline",
            install_hint="cargo install coraline  # (verify package name with `cargo search coraline`)",
            commands=_coraline_cmds(symbol),
            pre_clean=None,
        ),
    ]


# ── Sample + runner ─────────────────────────────────────────────────────────
@dataclass
class Sample:
    tool: str
    phase: str
    runs: list[float] = field(default_factory=list)
    err: str | None = None
    stdout_bytes: int = 0

    @property
    def median_s(self) -> float | None:
        return median(self.runs) if self.runs else None


def _resolve_binary(spec: CompetitorSpec) -> Path | None:
    """Return the runnable path, or None if the binary isn't on PATH.

    ``ecp`` is resolved via its absolute build path; competitors via
    ``shutil.which`` because they're expected to be system-installed.
    """
    if spec.name == "ecp":
        p = Path(spec.binary)
        return p if p.exists() else None
    found = shutil.which(spec.binary)
    return Path(found) if found else None


def _bench(tool: str, phase: str, cmd: list[str], cwd: Path, iterations: int) -> Sample:
    s = Sample(tool=tool, phase=phase)
    last_stdout = b""
    for _ in range(iterations):
        start = time.perf_counter()
        try:
            proc = subprocess.run(
                cmd, cwd=cwd, capture_output=True, timeout=CMD_TIMEOUT_S
            )
        except subprocess.TimeoutExpired:
            s.err = f"timeout after {CMD_TIMEOUT_S}s"
            return s
        elapsed = time.perf_counter() - start
        if proc.returncode != 0:
            s.err = (proc.stderr or proc.stdout).decode("utf-8", errors="replace")[:200].strip()
            return s
        s.runs.append(elapsed)
        last_stdout = proc.stdout
    s.stdout_bytes = len(last_stdout)
    return s


def _fmt(seconds: float | None) -> str:
    if seconds is None:
        return "    N/A"
    return f"{seconds * 1000:.1f} ms" if seconds < 1 else f"{seconds:.2f} s"


# ── Symbol probe (ecp-driven) ───────────────────────────────────────────────
def _probe_symbol(ecp: Path, corpus: Path) -> str | None:
    """Pick a Class-kind symbol that exists in ecp's graph. Competitors are
    then driven against the same name; if they don't index it, that's their
    coverage signal, not a benchmark bug."""
    proc = subprocess.run(
        [str(ecp), "cypher", "MATCH (a:Class) RETURN a.name LIMIT 10", "--format", "json", "--repo", str(corpus)],
        capture_output=True,
        text=True,
        timeout=60,
    )
    if proc.returncode != 0:
        return None
    try:
        rows = json.loads(proc.stdout).get("rows", [])
    except json.JSONDecodeError:
        return None
    candidates = [r for r in rows if isinstance(r, str) and len(r) >= 4 and r.replace("_", "").isalnum()]
    return candidates[0] if candidates else None


# ── Output ──────────────────────────────────────────────────────────────────
def _render_markdown(samples: list[Sample], corpus: Path, symbol: str | None, available: list[str], skipped: list[tuple[str, str]]) -> str:
    """Side-by-side phase × tool table. Missing competitors get a `_skipped_`
    line above the table so the absence is loud."""
    lines: list[str] = [
        "# Benchmark vs. peer-tier Rust competitors",
        "",
        f"Corpus: `{corpus}`",
        f"Probe symbol: `{symbol or '(unresolved)'}`",
        "",
    ]
    if skipped:
        lines.append("## Skipped (not on PATH)")
        lines.append("")
        for name, hint in skipped:
            lines.append(f"- `{name}` — install via `{hint}`")
        lines.append("")

    # Group samples (tool, phase) → Sample
    grid: dict[tuple[str, str], Sample] = {(s.tool, s.phase): s for s in samples}

    header = "| phase | " + " | ".join(available) + " |"
    sep = "|---|" + "|".join(["---"] * len(available)) + "|"
    lines.extend([header, sep])
    for phase in PHASES:
        cells: list[str] = [phase]
        for tool in available:
            s = grid.get((tool, phase))
            if s is None:
                cells.append("N/A")
            elif s.err:
                cells.append(f"FAIL ({s.err[:30]}…)" if len(s.err) > 30 else f"FAIL ({s.err})")
            elif s.median_s is None:
                cells.append("N/A")
            else:
                cells.append(_fmt(s.median_s))
        lines.append("| " + " | ".join(cells) + " |")
    lines.append("")
    lines.append(
        f"Generated by `python scripts/benchmark/benchmark_vs_competitors.py`. "
        f"Re-run to refresh."
    )
    return "\n".join(lines) + "\n"


def _render_svg(samples: list[Sample], available: list[str], out_path: Path) -> bool:
    """Bar chart per-phase grouped by tool. Returns True on success, False
    if matplotlib isn't importable (don't fail the whole script for the
    optional chart)."""
    try:
        import matplotlib  # type: ignore
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt  # type: ignore
    except ImportError:
        return False

    grid: dict[tuple[str, str], Sample] = {(s.tool, s.phase): s for s in samples}
    fig, ax = plt.subplots(figsize=(10, 5))
    bar_width = 0.8 / max(len(available), 1)
    x = list(range(len(PHASES)))
    for i, tool in enumerate(available):
        ys = []
        for phase in PHASES:
            s = grid.get((tool, phase))
            ys.append((s.median_s * 1000) if (s and s.median_s) else 0)
        ax.bar([xi + i * bar_width for xi in x], ys, bar_width, label=tool)
    ax.set_xticks([xi + bar_width * (len(available) - 1) / 2 for xi in x])
    ax.set_xticklabels(PHASES, rotation=20)
    ax.set_ylabel("median latency (ms)")
    ax.set_title("ecp vs. peer-tier code-intelligence tools")
    ax.legend()
    fig.tight_layout()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path)
    plt.close(fig)
    return True


# ── Main ────────────────────────────────────────────────────────────────────
def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--corpus", type=Path, default=DEFAULT_CORPUS, help="Target repo for indexing + queries")
    ap.add_argument("--iterations", type=int, default=3, help="Repeats per query phase (median wins)")
    ap.add_argument("--ecp-binary", type=Path, default=DEFAULT_ECP)
    ap.add_argument("--symbol", type=str, help="Explicit symbol for per-query phases (skips ecp-driven probe)")
    ap.add_argument("--output-md", type=Path, default=DEFAULT_OUTPUT_MD)
    ap.add_argument("--output-svg", type=Path, default=DEFAULT_OUTPUT_SVG)
    ap.add_argument("--no-plot", action="store_true", help="Skip the SVG chart")
    ap.add_argument("--json", type=Path, help="Dump raw results as JSON")
    args = ap.parse_args()

    if not args.corpus.is_dir():
        print(f"error: corpus is not a directory: {args.corpus}", file=sys.stderr)
        return 1
    if not args.ecp_binary.exists():
        print(f"error: ecp binary not at {args.ecp_binary}", file=sys.stderr)
        print(f"  build with: cargo build -p egent-code-plexus --bin ecp --release", file=sys.stderr)
        return 1

    # Symbol probe (ecp-driven so every competitor is asked about the same
    # name — competitor coverage gaps surface as that competitor's failure
    # rather than benchmark-design noise).
    symbol = args.symbol or _probe_symbol(args.ecp_binary, args.corpus)
    if not symbol:
        print("warning: no probe symbol found; per-query phases will be skipped", file=sys.stderr)

    specs = build_specs(symbol or "Unknown", args.ecp_binary)
    available: list[str] = []
    skipped: list[tuple[str, str]] = []
    for spec in specs:
        path = _resolve_binary(spec)
        if path:
            available.append(spec.name)
            print(f"  [ok] {spec.name}  ({path})")
        else:
            skipped.append((spec.name, spec.install_hint))
            print(f"  [skip] {spec.name} — not on PATH; install: {spec.install_hint}")

    if not available:
        print("error: no tools available — install at least ecp or one competitor", file=sys.stderr)
        return 1

    # Run benchmarks
    samples: list[Sample] = []
    for spec in specs:
        if spec.name not in available:
            continue
        binary = _resolve_binary(spec)
        assert binary is not None
        for phase in PHASES:
            cmd_builder = spec.commands.get(phase)
            if cmd_builder is None:
                # Tool doesn't expose this phase — recorded as N/A, not run.
                samples.append(Sample(tool=spec.name, phase=phase, err="unsupported phase"))
                continue
            # Drop the tool's cache before cold-index so the timing reflects
            # real first-run cost, not an idempotent no-op. Per-tool because
            # each indexer's cache lives in a different place.
            if phase == "cold-index" and spec.pre_clean is not None:
                clean_cmd = spec.pre_clean(binary, args.corpus)
                subprocess.run(clean_cmd, capture_output=True, timeout=60)
            cmd = cmd_builder(binary, args.corpus, symbol or "Unknown")
            print(f"→ {spec.name} {phase}")
            # cold-index always runs once (warm-cache makes repeats meaningless);
            # everything else uses --iterations.
            iters = 1 if phase == "cold-index" else args.iterations
            s = _bench(spec.name, phase, cmd, args.corpus, iters)
            samples.append(s)
            if s.err:
                print(f"  FAIL: {s.err}")
            else:
                print(f"  {_fmt(s.median_s)}  (stdout {s.stdout_bytes} B)")

    # Markdown output
    md = _render_markdown(samples, args.corpus, symbol, available, skipped)
    args.output_md.parent.mkdir(parents=True, exist_ok=True)
    args.output_md.write_text(md)
    print(f"\n→ wrote {args.output_md}")

    # SVG output (optional)
    if not args.no_plot:
        if _render_svg(samples, available, args.output_svg):
            print(f"→ wrote {args.output_svg}")
        else:
            print(f"  [skip-plot] matplotlib not importable; install via `pip install matplotlib`")

    # JSON dump
    if args.json:
        args.json.write_text(
            json.dumps({"corpus": str(args.corpus), "symbol": symbol, "samples": [asdict(s) for s in samples]}, indent=2)
        )
        print(f"→ wrote {args.json}")

    # Exit code: 0 if every available tool ran ≥1 phase successfully; 2 if
    # everything failed (covers "competitors installed but all crash" case).
    return 0 if any(s.runs for s in samples) else 2


if __name__ == "__main__":
    sys.exit(main())
