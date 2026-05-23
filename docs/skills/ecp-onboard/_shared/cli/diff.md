Edge-level resolver delta — binding tier-degradation (silent break), route / contract changes. For symbol blast-radius, use `ecp impact`

Usage: ecp diff [OPTIONS] --section <SECTION>

Options:
      --section <SECTION>
          Comma-separated section(s) to diff: bindings, routes, contracts, symbols, or all [possible values: bindings, routes, contracts, symbols, all]
      --baseline <BASELINE>
          Git ref to compare against: branch / tag / commit SHA / HEAD~N / PR/<n>. Required unless `--baseline-graph` is supplied (A-mode snapshot diff)
      --baseline-graph <BASELINE_GRAPH>
          A-mode: path to baseline `graph.bin` (skip git checkout + re-index). When set, requires `--current-graph` and restricts sections to those that read directly from graph.bin (routes / contracts / symbols)
      --current-graph <CURRENT_GRAPH>
          A-mode: path to current `graph.bin`. Required when `--baseline-graph` is supplied
      --format <FORMAT>
          Output format. Omit for the LLM-tuned default; pass `--format text|json|toon` for the alternative renderings
      --verbose
          List every change (text format only). Default truncates to top-10 per section
      --repo <REPO>
          Repository root path (defaults to current directory)
      --graph <GRAPH>
          Path to the graph.bin file [default: .ecp/graph.bin]
  -h, --help
          Print help
