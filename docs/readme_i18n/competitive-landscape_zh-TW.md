# 競品盤點

[English](../competitive-landscape.md)

最後盤點時間：2026-05-22。

「Rust + tree-sitter + MCP + 給 LLM agent 用的 code intelligence graph」這個賽道在
2026 年中異常擁擠 — 光是過去 3 個月就有 ≥15 個類似專案 push 到 GitHub。
本文記錄 `ecp` (egent-code-plexus) 在這個賽道中的定位、跟誰重疊、真正的差異化在哪。
當有新進入者或既有專案大改方向時更新。

## 最像的對照組（Rust-native、graph-first、MCP-wired）

| 專案 | Stars | License | 跟 ecp 重疊處 | 差異點 |
|---|---|---|---|---|
| [onur-gokyildiz-bhi/codescope](https://github.com/onur-gokyildiz-bhi/codescope) | 21 | MIT | Rust-native、MCP、「graph-first not embeddings-first」、ms 級 traversal、57 langs、9 agent 整合、Cargo.lock 用到 rkyv | 主 backend 是 SurrealDB、有 LSP mode + Web UI + daemon、沒 Process 抽象、沒 community detection |
| [cmillstead/codesight-mcp](https://github.com/cmillstead/codesight-mcp) | — | — | 66 langs tree-sitter、34 MCP tools、impact analysis | 重點在 retrieval 不在 graph 演算法層 |
| [postrv/narsil-mcp](https://github.com/postrv/narsil-mcp) | — | — | 32 langs、90 MCP tools、call graph、security scanning | 沒做 community / Process 抽象 |
| [basidiocarp/rhizome](https://github.com/basidiocarp/rhizome) | — | — | tree-sitter + LSP 雙 backend、sub-ms parse | 更像 LSP wrapper，沒 graph storage 層 |
| [kuberstar/qartez-mcp](https://github.com/kuberstar/qartez-mcp) | — | — | 37 langs（tree-sitter + regex fallback）、project map、symbol search、impact analysis | 沒做 community / Process 抽象 |
| [greysquirr3l/coraline](https://github.com/greysquirr3l/coraline) | 10 | Apache-2.0 | 28 langs、MCP、號稱 sub-second indexing | SQLite backend、沒 community detection |
| [shaharia-lab/code-navigator](https://github.com/shaharia-lab/code-navigator) | 5 | MIT | 「compressed graph」for AI agents、impact analysis | 還在早期 |
| [Jakedismo/codegraph-rust](https://github.com/Jakedismo/codegraph-rust) | 754 | 不明（README 寫 MIT/Apache 但 repo 根目錄沒 LICENSE 檔） | 14 crates、Rust + MCP | 5 個月沒 push（last 2025-12-20）、SurrealDB、重點是 agent framework（Rig + LATS + Reflexion）、不做 community |

## 相鄰賽道（部分重疊、定位不同）

| 專案 | 重點 |
|---|---|
| [github/stack-graphs](https://github.com/github/stack-graphs) | 877★。GitHub 官方 Rust tree-sitter 跨檔符號解析。只解 cross-reference，沒 community detection、沒 Process。 |
| [probelabs/probe](https://github.com/probelabs/probe) | ripgrep 速度 + tree-sitter AST、semantic search。沒 graph storage。 |
| [faxioman/code-sage](https://github.com/faxioman/code-sage) | BM25 + 向量 + tree-sitter chunking。Semantic search，不是 graph。 |
| [flupkede/codesearch](https://github.com/flupkede/codesearch) | hybrid 向量 + BM25 + tree-sitter chunking。 |
| [rustkit-ai/semtree](https://github.com/rustkit-ai/semtree) | tree-sitter + embeddings + RAG multi-backend。 |
| [hankh95/nusy-codegraph](https://github.com/hankh95/nusy-codegraph) | Arrow-native code object storage。值得追蹤的不同 storage 角度。 |

## ecp 真正獨有的地方

整個賽道基本收斂到同樣的 baseline stack：**Rust + tree-sitter + MCP + sub-second-indexing 宣稱**。
讓 ecp 拉開差距的：

| 賽道共通項（同質化） | ecp 的具體賭注 |
|---|---|
| tree-sitter parse 28-66 種語言 | **Leiden community detection → `NodeKind::Process` 語意抽象**。給 LLM「execution flow」級別的語意，不只 callee/caller。盤點的所有專案中沒有第二家做這件事。 |
| Impact analysis (callers / callees) | **Deterministic seeded 輸出**。同 corpus + 同 seed → bit-identical `graph.bin`。透過 `LeidenConfig::default().seed = 0xc0de` 與 XorShift64 RNG 固定。讓我們可以對 `ref-gitnexus` 跑 A/B oracle 重現比對。 |
| MCP tool 包裝 | **Zero-copy rkyv mmap 作為主要存儲**。codescope 也用 rkyv，但只是 SurrealDB 上的 transitive dep。我們把它當作 on-disk 主格式 — query path 上沒有 DB 引擎。 |
| BM25 / 向量混合 | **Cypher 子集** 圖查詢。賽道內少有人暴露 graph-query 語意；多數只給 JSON tool-call wrapper。 |

## 不該抄的選擇

下列是賽道上的常見選擇，但跟 ecp 的 per-query latency / cold-ingest 目標衝突：

- **SurrealDB / SQLite 當主要 backend**（codescope、Jakedismo、coraline） —
  query path 走 DB 引擎，跟我們的 <30 ms / query 與 <5 s cold-ingest 目標衝突。
- **AI / RAG / embedding pipeline 整合進 core**（Jakedismo、semtree、code-sage） —
  把 ecp 跟 LLM call 延遲、廠商 API 綁死。我們的核心競爭力是 deterministic 不是 fuzzy。
- **LSP 當 default backend**（rhizome） — 光是 LSP cold start（每語言 3-10s）
  就吃掉我們的 cold-ingest 目標。LSP 只能是 opt-in 加強層。
- **MCP tool 數量競賽**（narsil 90 tools、codesight 34、codescope 32） —
  tool 多 ≠ tool 好。每個 tool 都是一份 LLM 要讀的 contract 文件；
  表面積只該隨著實際需求成長。

## 可以借鑒的（小、具體、opt-in）

| 從誰 | 想法 | 成本 |
|---|---|---|
| codescope | `codescope insight` 風格的 per-repo + hourly MCP tool 使用率 telemetry — 給 user 看到 agent 實際呼了哪些 tool | 低（純 telemetry，不動演算法） |
| codescope、Jakedismo | LSP bridge **作為 opt-in feature** — 解決 tree-sitter 解不開的 case（C++ template、Java generic） | 中。必須 feature-gated 讓 default path 保住 cold-ingest budget。 |
| nusy-codegraph | Arrow-native storage 角度 — 跟 rkyv 一樣零拷貝但跨語言生態大（Python pandas 可 mmap 直讀）。只有 Python wheel binding 變成需求時才相關 | 高；先放著。 |
| codescope、coraline | 公開 sub-second-indexing benchmark **當標準對照** — 同 corpus 並列數字 | 低工程量、中等行銷後續。 |

## 對 roadmap 的啟示

差異化在 **「演算法層產生語意抽象」**（Leiden → Process），不在 tool 數、語言數、agent 整合廣度。
繼續深挖這條軸的 ROI 高於追趕 LSP / embedding / agent-framework 等已經是 table-stakes 的功能。
