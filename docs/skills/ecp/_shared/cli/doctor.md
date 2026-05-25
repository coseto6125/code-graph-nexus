# ecp doctor

Environment health check. Aggregates independent checks into one report so a
drifted setup (stale skills, stale graph, outdated host config) surfaces in a
single command instead of failing silently mid-workflow.

## Usage

```bash
ecp doctor          # read-only report
ecp doctor --fix    # also reinstall stale skills + rebuild a stale index
```

## Checks

| Check | Pass | Warn / Fail |
|---|---|---|
| `skill:<name>` | installed copy matches repo source | stale (source differs) → Warn; not installed → Warn |
| `index` | graph fresher than working tree | stale → Warn; missing → Fail |
| `host:<tool>` | integrated, or optional and absent | config outdated → Warn |
| `config:ecp-home` / `config:claude-dir` | path exists and writable | missing / read-only → Warn |

Exit code is non-zero when any check is **Fail** (e.g. missing index), so CI can
gate on `ecp doctor`. Warnings alone do not fail the run.

## `--fix`

Reruns the remediation for fixable checks in place:
- **skills**: equivalent to `ecp admin claude install skills <name>`.
- **index**: equivalent to `ecp admin index --repo .`.

Host-integration and config findings are **report-only** — they print a
remediation hint but `--fix` never rewrites user-owned host configs.

## Related: install diff

`ecp admin claude install skills <target>` now always prints a diff of what it
changes (added / removed / modified files, with a warning when an installed
file looks hand-edited). `--dry-run` prints that diff without writing —
the same engine `doctor` uses to detect skill staleness.
