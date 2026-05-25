# Wave 1 Language Baseline Stats

> **Note:** The 2026-05-14 dev-profile numbers are superseded by this release-profile recapture against pinned commits.

Captured: 2026-05-25  
ecp binary: `target/release/ecp` (release profile)  
Command: `ecp admin index --repo .sample_repo/<lang> --force`

## Pinned Commits

| Lang       | Pinned SHA                               |
|------------|------------------------------------------|
| lua        | 359f0e2742f51ca77801b513ec91eb9029de8de4 |
| solidity   | cd05883078060e0cd8a7bd36636944570dbe1722 |
| bash       | 854c4ee02d033dbdd22183fb164ff84373c851aa |
| zig        | 569bba10f22afd4ea1815416b546a8065905f820 |
| crystal    | 5023c21195cff9b0fca700bc582911b179c2add5 |
| dockerfile | 2353f0380c24944616282f94544cec2d462e1e2a |
| move       | 47de220ebaa3e70bc5547911045971961395c233 |

## Baseline Results

| Lang       | Upstream Repo                              | Files | Nodes | Scan | Analyze | Total  | Status |
|------------|--------------------------------------------|-------|-------|------|---------|--------|--------|
| lua        | kikito/middleclass (depth=1)               | 11    | 173   | N/A  | N/A     | 0.04s  | OK     |
| solidity   | OpenZeppelin/openzeppelin-contracts (d=1)  | 702   | 7245  | N/A  | N/A     | 0.17s  | OK     |
| bash       | Bash-it/bash-it (depth=1)                  | 344   | 5014  | N/A  | N/A     | 0.13s  | OK     |
| zig        | karlseguin/http.zig (depth=1)              | 31    | 2654  | N/A  | N/A     | 0.06s  | OK     |
| crystal    | kemalcr/kemal (depth=1)                    | 74    | 470   | N/A  | N/A     | 0.06s  | OK     |
| dockerfile | docker-library/postgres (depth=1)          | 70    | 1187  | N/A  | N/A     | 0.05s  | OK     |
| move       | aptos-labs/aptos-core (sparse, d=1)        | 458   | 9501  | N/A  | N/A     | 0.24s  | OK     |

> Scan / Analyze columns are N/A: the release binary emits only aggregate `elapsed=Xs` (no sub-phase breakdown). The dev-profile binary emitted per-phase timings; the release binary does not.

## Notes

### zig — OK (2654 nodes, was PARTIAL in 2026-05-14)

The zig parser has been implemented since the 2026-05-14 snapshot. All 31 `.zig` files from
`karlseguin/http.zig` are now parsed with 2654 nodes extracted. The PARTIAL status is resolved.

### move — sparse checkout

Aptos-core is ~3 GB total; sparse checkout of `aptos-move/framework/` reduces
it to ~15 MB on disk. The 458 files include `.move` sources only from that
subtree.

### Disk usage (.sample_repo/wave-1 only)

| Dir          | Size  |
|--------------|-------|
| lua          | 324 K |
| solidity     | 22 M  |
| bash         | 5.7 M |
| zig          | 928 K |
| crystal      | 948 K |
| dockerfile   | 1.1 M |
| move         | 15 M  |
| **Total**    | **~46 M** |
