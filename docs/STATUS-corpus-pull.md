# STATUS · Day 21 · Corpus Pull 真活

> 日期：2026-05-29
> 对应 commit：[本 commit · Day 21 corpus pull]
> 对应 feature log：[`feedback/features/rag4arkui-core/23-2026-05-29-corpus-pull.md`](../feedback/features/rag4arkui-core/23-2026-05-29-corpus-pull.md)
> 对应 meta：[`feedback/meta/10-2026-05-29-corpus-pull-deps.md`](../feedback/meta/10-2026-05-29-corpus-pull-deps.md)
> 上一阶段：[`STATUS-pre-existing-fixes.md`](STATUS-pre-existing-fixes.md)
> 下一阶段：`STATUS-onnx-real.md`（Day 20c onnx 真活）或 `STATUS-mdbook-doc.md`（Day 22 文档站）

> 🎯 **里程碑**：**Day 21 完成 · `arkui-rag corpus pull` 真活 · Week 6 进度 4/4 ⭐**

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `crates/arkui-rag-cli/src/main.rs` | + `CorpusOp::Pull` 变体 · + `cmd_corpus_pull()` 实装 ~110 行 |
| `crates/arkui-rag-cli/Cargo.toml` | + `ureq` / `flate2` / `tar` optional deps · + `corpus-pull` feature · `full` 加 corpus-pull |
| `crates/Cargo.toml` | workspace 加 ureq / flate2 / tar 版本声明（ureq 配 rustls 内置 + 无 OpenSSL 依赖） |
| `scripts/release-local.sh` | DEFAULT_FEATURES 加 corpus-pull |
| `.github/workflows/release.yml` | FEATURES env 加 corpus-pull |
| `docs/RELEASE.md` | feature 表加 corpus-pull 行 |
| `README.md` | Download 章节加 `corpus pull` 一键拉取示例 |

### Binary 影响

| 维度 | Day 20c 完成时 | Day 21 完成时 |
|---|---|---|
| Release binary | 6.7 MB | **11 MB**（+ ureq + tar + flate2 ~4 MB） |
| Release tarball | 2.9 MB | **4.1 MB**（gzip ~62%） |
| 默认 features 项数 | 5 | **6**（+ corpus-pull） |
| CLI 子命令数 | 4 | **5**（+ `corpus pull`） |

---

## 输入契约

### CLI 用法

```bash
# 一键从默认 URL 拉取（GitHub Releases corpus-v0.0.1 tarball）
arkui-rag corpus pull
arkui-rag corpus pull --target ./corpus/official            # 自定义目标

# 自定义 URL（gitcode mirror / 自建镜像 / 私有 corpus）
arkui-rag corpus pull --url https://example.com/my-corpus.tar.gz --target ./corpus/custom

# 离线场景：跳过 HTTP 从本地文件解压
arkui-rag corpus pull --from-file ./downloaded.tar.gz --target ./corpus/official

# 兼容不同 tarball 结构（默认 strip 1 段 · 即剥外层 wrap 目录）
arkui-rag corpus pull --strip-components 0   # tarball 内已经是裸文件
arkui-rag corpus pull --strip-components 2   # tarball 内有 2 层 wrap 目录

# 覆盖非空目标目录
arkui-rag corpus pull --force --target ./corpus/official
```

### Tarball 格式约定

```
arkui-rag-corpus-vX.Y.Z.tar.gz
└── arkui-rag-corpus-vX.Y.Z/           <-- strip_components=1 剥掉
    ├── README.md                      # 元信息（来源、版本、文档数）
    ├── components/
    │   ├── list.md
    │   └── ...
    └── api/
        └── ...
```

---

## 输出契约

### `corpus pull` 成功输出

```
✅ corpus 拉取完成
   来源     : <URL 或 file 路径>
   目标     : ./corpus/official
   大小     : 0.01 MB
   文件数    : 8
   strip    : 1 段

下一步：
   arkui-rag index --source ./corpus/official --index-path ./corpus/official/index.json --bm25 tantivy
```

### 错误响应

| 错误 | 触发条件 |
|---|---|
| `目标目录非空：... 加 --force 覆盖` | target 已存在且非空且未指定 --force |
| `HTTP GET 失败：...` | 网络问题 / 404 / TLS 问题 |
| `拒绝解压：路径越界 ... 不在 ... 内` | 恶意 tarball 含 `../` 类 path traversal |
| `corpus pull 需要 corpus-pull feature 启用` | binary 编译时未带 corpus-pull feature |

---

## 验证手段

### 用户手动

```bash
# 1. 编译带 corpus-pull
make release-local                            # 默认 features 已含 corpus-pull
# 或：cargo build --release --features corpus-pull,tantivy -p arkui-rag-cli

# 2. 自打 test tarball
mkdir -p /tmp/test/staging/arkui-rag-corpus-v0.0.1/official
cp some-docs/*.md /tmp/test/staging/arkui-rag-corpus-v0.0.1/official/
cd /tmp/test/staging && tar -czf /tmp/test/corpus.tar.gz arkui-rag-corpus-v0.0.1

# 3. 端到端 pull
mkdir -p /tmp/test/target
arkui-rag corpus pull --from-file /tmp/test/corpus.tar.gz --target /tmp/test/target

# 4. 验证
find /tmp/test/target -name '*.md'           # 应列出解压后的文件
arkui-rag index --source /tmp/test/target --index-path /tmp/test/target/index.json --bm25 tantivy
arkui-rag query --text "..." --index-path /tmp/test/target/index.json --bm25 tantivy -k 3
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `cargo check -p arkui-rag-cli --features corpus-pull` | 编译校验 | ✅ |
| 端到端 pull → index → query（本地 test tarball） | 业务链路 | ✅ |
| `make release-local`（含 corpus-pull） | 打包产出 | ✅ |
| **M-STATUS-PER-ROUND** Round 23 + STATUS-corpus-pull 配套 | 元规则 | ✅ |
| **ROADMAP 维护约定（第 12 次实战）** | 当前位置 + Week 6 进度 + 已完成表 | ✅ |

### 暂未自动化（明确缺口）

- ❌ `--url` HTTP 路径（默认 URL 是占位 · 用户首次推 corpus tarball release 才能跑）
- ❌ macOS tar 的 `._*` AppleDouble 噪声跳过
- ❌ checksum 校验（SHA256 比对）
- ❌ 下载进度条（indicatif）
- ❌ `corpus model-pull` 真活（BGE-M3 ONNX · Day 21b）

---

## 与上一阶段（STATUS-pre-existing-fixes）的关联性

### 增量

| 维度 | Day 20c 完成时 | 本轮（Day 21）后 |
|---|---|---|
| CLI 子命令 | 4 | **5**（+ corpus pull） |
| 默认 release features | 5 项 | **6 项** |
| Release binary | 6.7 MB | 11 MB |
| Release tarball | 2.9 MB | 4.1 MB |
| 用户首次接入流程 | 下 binary → 手动投放文档 → index → query | 下 binary → **`corpus pull`** → index → query |
| Week 6 进度 | 3/4 | **4/4** ⭐ |

### 兼容性

- ✅ 无破坏性变更（CLI 仅新增子命令 · 现有 list / model-pull 不动）
- ✅ `corpus-pull` feature 默认启用但可关闭（极小 binary 场景）
- ✅ 现有 ci.yml（PR 校验）不动
- ✅ release.yml 与本地 scripts 默认 features 同步

---

## 完成度 / 下一阶段

### Day 21 完成度

| 项 | 状态 |
|---|---|
| `CorpusOp::Pull` 子命令 + 5 个参数（url/from-file/target/force/strip-components） | ✅ |
| `cmd_corpus_pull()` 实装 ~110 行 | ✅ |
| ureq + tar + flate2 接入（feature gated） | ✅ |
| Path traversal 安全检查 | ✅ |
| 500 MB 下载上限 | ✅ |
| 默认 release features 加 corpus-pull | ✅ |
| scripts/release.yml 同步 | ✅ |
| docs/RELEASE.md + README 更新 | ✅ |
| 端到端实测（pull → index → query） | ✅ |
| `--url` 真活验证（默认 URL 仍占位 · 待用户推 corpus tarball release） | ⏳ |
| `corpus model-pull` 真活（BGE-M3 ONNX） | ⏳ Day 21b |
| checksum / 进度条 / AppleDouble 跳过 | ⏳ 细节优化 |

### 6 周路线图达成度

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| **Week 6: 发布 + 文档站 + 评估报告** | **4/4** ✅（评估 ✓ · 本地 release ✓ · CI matrix ✓ · **corpus pull ✓**） |

**总完成度估算：~85%**（Week 6 完成 4/4，仅 1.0 release page + mdBook 待做）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 22 mdBook 文档站 + 1.0 release** | 公开发布 · MVP 完整收尾 | 1-2 commit |
| 🟢 **Day 20c onnx 真活**（BGE-M3 真语义 embedding） | 解锁真 RAG · 不再是 mock | 2-3 commit |
| 🟡 **Day 21b corpus model-pull 真活** | BGE-M3 模型自动下载 · 共用 cmd_corpus_pull 基础设施 | 1 commit |
| 🟡 task #81 升 lancedb 0.10 → 0.20+ | 解锁向量库 · 1k+ chunks scale | 1-2 commit |
| 🟡 用户准备 corpus tarball 推 GitHub Release `corpus-v0.0.1` | 让 `corpus pull` 默认 URL 真活 | 0.5 commit |
| ⚪️ 用户首次推 v0.0.2 tag 验证 CI matrix release.yml | 锦上添花 · 验完挂 status badge | 0.5 commit |

**Agent 推荐**：**Day 22 mdBook 文档站 + 1.0 release**（公开发布 MVP 最后一步）或 **Day 20c onnx 真活**（让 RAG 真有语义检索而不是 mock）。两者均有高价值，看用户取舍。

### 重要的"非完成"项

- ❌ Day 22 mdBook 文档站
- ❌ Day 20c onnx 真活（BGE-M3）
- ❌ Day 21b corpus model-pull 真活
- ❌ task #81 lancedb 主版本升级
- ❌ ArkTS struct method extraction（custom grammar）
- ❌ release.yml 第一次实跑验证
