Cypher query escape hatch

Usage: ecp cypher [OPTIONS] [QUERY]

Arguments:
  [QUERY]
          The Cypher query string. Supports a read-only subset of openCypher:
          
          - Multi-hop patterns: (a)-[:Calls]->(b)-[:Calls]->(c) - Variable-length:    (a)-[:Calls*1..3]->(b) - Label alternation:  (a:Function|Method) - WHERE:              =, <>, <, <=, >, >=, AND, OR, NOT, IN, =~, CONTAINS, STARTS WITH, ENDS WITH - Properties (node):  name, kind, filePath, uid, ownerClass, content, is_test, is_async, is_static, is_abstract, is_generator, is_extern, visibility, decorators - Properties (edge):  confidence, reason, rel_type - Aggregation:        COUNT(*), COUNT(DISTINCT x), SUM/MIN/MAX/AVG, COLLECT - Pipeline:           WITH ... [WHERE ...], OPTIONAL MATCH, UNION [ALL] - Output shaping:     RETURN [DISTINCT], ORDER BY, SKIP, LIMIT
          
          Cypher operates on a single graph; --repo must resolve to one repo.

Options:
      --query <QUERY>
          Named alias for the positional QUERY argument

      --repo <REPO>
          Repository to query. Cypher operates on a single graph (single-repo only). If --repo resolves to multiple repos, an error is returned

      --format <FORMAT>
          Output format. Omit for the LLM-tuned default; explicit `--format toon|json|text` for the neutral / round-trippable / human paths

      --graph <GRAPH>
          Path to the graph.bin file
          
          [default: .ecp/graph.bin]

  -h, --help
          Print help (see a summary with '-h')
