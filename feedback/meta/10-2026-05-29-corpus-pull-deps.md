# 10 — corpus-pull-deps

> 日期：2026-05-29
> 触发：用户指令「请继续」（执行 Day 21 corpus pull · 推荐顺序中的下一步）
> 类型：工程结构调整（workspace deps + cli features + scripts/CI 默认值 同步更新）

---

## 用户提出的要求

按 agent 推荐顺序，Day 21 corpus pull 真活：让用户拿到 release binary 后能 `arkui-rag corpus pull` 自动获取默认 corpus，不用手动投放文档。

## Agent 给出的修改建议

| 改动 | 类别 | 内容 |
|---|---|---|
| `crates/Cargo.toml` | meta（workspace 顶层） | + ureq / flate2 / tar 版本声明 |
| `crates/arkui-rag-cli/Cargo.toml` | business（crate manifest） | + optional deps · + `corpus-pull` feature · `full` 加 corpus-pull |
| `crates/arkui-rag-cli/src/main.rs` | business（Rust 源） | + CorpusOp::Pull · + cmd_corpus_pull() ~110 行 |
| `scripts/release-local.sh` | meta（脚本默认值） | DEFAULT_FEATURES 加 `corpus-pull` |
| `.github/workflows/release.yml` | meta（CI workflow yml） | FEATURES env 加 `corpus-pull` |
| `docs/RELEASE.md` | business（用户文档） | feature 表加 corpus-pull 行 |
| `README.md` | meta（项目入口） | Download 章节加 `corpus pull` 一键拉取示例 |

### 关键决策

1. **HTTP 客户端选 ureq**（不选 reqwest）：体积小 · rustls 内置 · 无 tokio runtime 依赖 · sync API + spawn_blocking 桥接
2. **feature gated**：`corpus-pull` 单独 feature · 不想要 corpus pull 的极小 release 可以不带
3. **`corpus-pull` 进默认 release features**：用户向 CLI 几乎都需要
4. **同步 scripts/release-local.sh + release.yml**：避免本地与 CI 默认 features 漂移

## 多轮互动

无 —— 用户给出「请继续」指令后 agent 直接按推荐顺序执行。

## 实际改动

- **接口变化**：新增 CLI 子命令 `arkui-rag corpus pull --url|--from-file [--target] [--force] [--strip-components N]`
- **规则变化**：无（AGENTS.md 不动）
- **文件变化**：
  - 修：`crates/Cargo.toml`、`crates/arkui-rag-cli/Cargo.toml`、`crates/arkui-rag-cli/src/main.rs`
  - 修：`scripts/release-local.sh`、`.github/workflows/release.yml`
  - 修：`docs/RELEASE.md`、`README.md`
- **配置变化**：默认 release features 5 项 → 6 项

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| `arkui-rag corpus pull --from-file <path>` 本地解压链路 | ✅ |
| `arkui-rag corpus pull --url <URL>` HTTP 下载链路 | ✅ 实装 · ⏳ 待用户首推 corpus tarball release 验证 |
| Path traversal 安全检查 | ✅ |
| 500 MB 下载上限 | ✅ |
| `--strip-components N` 兼容不同 tarball 顶层结构 | ✅（默认 1） |
| 默认 release features 包含 corpus-pull | ✅ |
| docs/RELEASE.md feature 表 + README Download 章节 | ✅ |

### 前后对比

| 维度 | Day 20c 完成时 | 本轮（Day 21）完成时 |
|---|---|---|
| CLI 子命令 | 4（serve / index / query / eval + corpus list/model-pull） | **5**（+ `corpus pull`） |
| 默认 release features | 5 项 | **6 项**（+ corpus-pull） |
| Release binary 大小 | 6.7 MB | **11 MB**（+ ureq + tar + flate2 ~4 MB） |
| Release tarball 大小 | 2.9 MB | **4.1 MB** |
| 用户首次使用流程 | 下 binary → **手动投放文档** → index → query | 下 binary → **`corpus pull`** → index → query |

### 实测验证

端到端 4 步全过：

```
$ cargo build --release -p arkui-rag-cli --features corpus-pull,tantivy
   Finished `release` profile [optimized] target(s) in 43.95s ✅

$ arkui-rag corpus pull --from-file test-tarball.tar.gz --target /tmp/corpus-target-d21
✅ corpus 拉取完成 · 文件数 8 · strip 1 段

$ find /tmp/corpus-target-d21 -name '*.md'
/tmp/corpus-target-d21/official/mapping-list.md
/tmp/corpus-target-d21/official/mapping-state.md
/tmp/corpus-target-d21/official/mapping-async.md ✅

$ arkui-rag index --source /tmp/corpus-target-d21 --bm25 tantivy --index-path ...
✅ files=3 chunks=22 elapsed_ms=125

$ arkui-rag query --text "@State 双向绑定" --bm25 tantivy -k 2
✅ Top-2 命中 mapping-state.md（L24-34 "状态选择决策"）
```

### 残留 / 下一轮处理

- [ ] **关键**：用户准备 corpus tarball 推 GitHub Release `corpus-v0.0.1` · 验证 `--url` 默认路径
- [ ] macOS tar 的 `._*` AppleDouble 噪声（解压时跳过 · 细节优化）
- [ ] 进度条 indicatif（大 tarball 时 UX 提升）
- [ ] checksum 校验（SHA256 比对 · 防中间人）
- [ ] `corpus model-pull` 真活（共用 cmd_corpus_pull 基础设施 · Day 21b）
- [ ] task #81 升 lancedb 主版本
- [ ] Day 22 mdBook 文档站 + 1.0 release
- [x] workspace deps + cli feature 接入
- [x] CLI Pull 子命令实装 + 安全检查
- [x] scripts/release-local + release.yml 默认 features 同步
- [x] docs/RELEASE.md + README 更新
- [x] 端到端实测（pull → index → query）
