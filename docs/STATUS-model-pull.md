# STATUS · Day 21b · model-pull 真活

> 日期：2026-05-29
> 对应 commit：[本 commit · Day 21b model-pull]
> 对应 feature log：[`feedback/features/rag4arkui-core/25-2026-05-29-model-pull.md`](../feedback/features/rag4arkui-core/25-2026-05-29-model-pull.md)
> 上一阶段：[`STATUS-mdbook-doc.md`](STATUS-mdbook-doc.md)
> 下一阶段：`STATUS-onnx-real.md`（推荐 · 用真模型跑 onnx feature 端到端）

> 🎯 **里程碑**：**`arkui-rag corpus model-pull --name bge-m3` 真活 · 共用 corpus pull 基础设施 · 模型 1 步到位**

---

## 当前状态

| 模块 | 变化 |
|---|---|
| `crates/arkui-rag-cli/src/main.rs` | `CorpusOp::ModelPull` 从 stub → 真活 · 重构 `cmd_corpus_pull` 抽出 `download_and_extract` shared helper · 新增 `cmd_corpus_model_pull` + `default_model_url` + `default_model_target` |
| `mdbook/src/usage/corpus.md` | + model-pull 章节 · 默认 URL 路由表 · tarball 格式约定 |

### 影响范围

- CLI 子命令数：5（不变 · `model-pull` 从 stub 升级到真活）
- 默认 release features：6（不变 · 已含 `corpus-pull`）
- Release binary：~11 MB（不变 · 仅复用已有 ureq/tar/flate2 deps）
- 净代码量：+~200 行（120 新增 + ~80 重构）

---

## 输入契约

### CLI 用法

```bash
# 一键拉默认 BGE-M3（按 name 路由 + 默认 target ~/.arkui-rag/models/bge-m3/）
arkui-rag corpus model-pull --name bge-m3

# 拉 reranker
arkui-rag corpus model-pull --name bge-reranker-v2-m3

# 自定义 URL（gitcode mirror / HuggingFace 直链 / 自建镜像）
arkui-rag corpus model-pull \
    --name custom-emb \
    --url https://example.com/my-model.tar.gz \
    --target ~/.arkui-rag/models/custom-emb

# 离线场景：本地 tarball 解压
arkui-rag corpus model-pull --name bge-m3 --from-file ./bge-m3-onnx-v1.tar.gz

# 覆盖已存在文件
arkui-rag corpus model-pull --name bge-m3 --force

# 自定义 strip-components
arkui-rag corpus model-pull --name bge-m3 --strip-components 0
```

### 已知模型 → 默认 URL 路由

| `--name` | 默认 URL |
|---|---|
| `bge-m3` | `https://github.com/keerecles/RAG4ArkUI/releases/download/models-v1/bge-m3-onnx-v1.tar.gz` |
| `bge-reranker-v2-m3` | `https://github.com/keerecles/RAG4ArkUI/releases/download/models-v1/bge-reranker-v2-m3-onnx-v1.tar.gz` |
| 其它 | bail · 用 `--url` 自定义 |

加新模型只改 `default_model_url()` 一处。

### Tarball 格式约定（同 Day 21）

```
bge-m3-onnx-v1.tar.gz
└── bge-m3-onnx-v1/                          <-- strip_components=1 剥掉
    ├── model/model.onnx                     # 主模型（或 model.fp16.onnx）
    ├── tokenizer.json                       # HuggingFace tokenizer 标准格式
    ├── special_tokens_map.json
    └── config.json
```

---

## 输出契约

### model-pull 成功输出

```
✅ 模型拉取完成
   model    : bge-m3
   来源     : <URL 或 file 路径>
   目标     : /Users/leo/.arkui-rag/models/bge-m3
   大小     : 0.00 MB
   文件数    : 6
   strip    : 1 段

下一步（用真模型跑 index/query · 需要 onnx feature）：
   arkui-rag index ... --embedder onnx --model-path /Users/leo/.arkui-rag/models/bge-m3
   arkui-rag query ... --embedder onnx --model-path /Users/leo/.arkui-rag/models/bge-m3
```

### 错误响应

| 错误 | 触发条件 |
|---|---|
| `未知模型名: X · 已知: bge-m3 / bge-reranker-v2-m3 · 或加 --url 自定义` | `--name` 不在路由表 + 未传 `--url` |
| `HOME / USERPROFILE 环境变量都未设置 · 用 --target 显式指定目录` | 极少见（容器 / CI 没 home env） |
| `目标目录非空：... 加 --force 覆盖` | 目录已存在内容 + 未 --force |
| `HTTP GET 失败：...` | 网络 / 404 / TLS |
| `拒绝解压：路径越界 ...` | 恶意 tarball 含 `../` |

---

## 验证手段

### 用户手动

```bash
# 1. 编译
make release-local
# 或 cargo build --release --features corpus-pull,tantivy -p arkui-rag-cli

# 2. 自打 fake-bge-m3 tarball
mkdir -p /tmp/fake-model/bge-m3-test/model
echo "fake-onnx-bytes" > /tmp/fake-model/bge-m3-test/model/model.onnx
echo '{}' > /tmp/fake-model/bge-m3-test/tokenizer.json
cd /tmp/fake-model && tar -czf /tmp/test.tar.gz bge-m3-test

# 3. 端到端 model-pull
arkui-rag corpus model-pull --name bge-m3 --from-file /tmp/test.tar.gz --target /tmp/out

# 4. 验证
find /tmp/out -name '*.onnx' -o -name '*.json'

# 5. （需 onnx feature 编译）用真模型跑 index/query
cargo build --release --features corpus-pull,tantivy,onnx -p arkui-rag-cli
arkui-rag index --source ./corpus/official --embedder onnx --model-path /tmp/out ...
```

### 自动化

| 手段 | 范围 | 状态 |
|---|---|---|
| `cargo check -p arkui-rag-cli --features corpus-pull` | 编译校验 | ✅ |
| `cargo build --release` 重构无 regression | 重构稳定 | ✅ |
| fake tarball 端到端 model-pull | 业务链路 | ✅ |
| Regression corpus pull | 共用 helper 不破坏 Day 21 | ✅ |
| **M-STATUS-PER-ROUND** Round 25 + STATUS-model-pull | 元规则 | ✅ |
| **ROADMAP 维护约定（第 14 次实战）** | 当前位置 + 已完成表 | ✅ |

### 暂未自动化（明确缺口）

- ❌ `--url` HTTP 路径（默认 URL 当前 404 · 用户首推 models-v1 release 后真活）
- ❌ 真 BGE-M3 ONNX 端到端（需用户准备模型 + 跑 onnx feature）
- ❌ SHA256 checksum 校验
- ❌ 下载进度条 indicatif
- ❌ `corpus model-list`

---

## 与上一阶段（STATUS-mdbook-doc）的关联性

### 增量

| 维度 | Day 22 完成时 | 本轮（Day 21b）后 |
|---|---|---|
| Crate 数 | 9 | 不变 |
| CLI 子命令 | 5 | 不变（model-pull 从 stub → 真活） |
| 默认 release features | 6 项 | 不变 |
| `corpus model-pull` 状态 | ⏳ stub（打印 git clone 指南） | ✅ **真活** |
| 共享 helper | 无（cmd_corpus_pull 单用） | **`download_and_extract`** |
| 默认模型源 | 无 | 路由表 2 个模型 |
| 默认模型目录 | 无 | `~/.arkui-rag/models/<name>/` |

### 兼容性

- ✅ 无破坏性变更（model-pull 子命令保留 · 参数扩展）
- ✅ regression corpus pull 验证通过
- ✅ binary 体积不变
- ✅ 现有 ci.yml / release.yml / book.yml 不动

---

## 完成度 / 下一阶段

### Day 21b 完成度

| 项 | 状态 |
|---|---|
| `cmd_corpus_model_pull` 真活实装 | ✅ |
| `default_model_url` 路由表 | ✅ |
| `default_model_target` HOME/USERPROFILE | ✅ |
| 抽 `download_and_extract` 共享 helper | ✅ |
| `CorpusOp::ModelPull` 6 参数（name/url/target/force/from-file/strip-components） | ✅ |
| mdbook/src/usage/corpus.md model-pull 章节 | ✅ |
| 端到端 fake tarball 验证 | ✅ |
| Regression corpus pull 验证 | ✅ |
| `--url` HTTP 真活（默认 URL 占位） | ⏳ 用户准备 models-v1 release |
| 真 BGE-M3 ONNX 端到端 | ⏳ Day 20c |
| SHA256 / 进度条 / model-list | ⏳ 细节优化 |

### 6 周路线图达成度

| 章节 | 状态 |
|---|---|
| Week 1: Rust 骨架 + tree-sitter + LanceDB + Tantivy + BGE-M3 | **7/7** ✅ |
| Week 2: 混合检索 + Reranker + HyDE + 评估集 | **4/4** ✅ |
| Week 3: HTTP + MCP + CLI | **3/3** ✅ |
| Week 4: 协议层（HTTP + MCP + LSP） | **3/3** ✅ ⭐ |
| Week 5: Claude Code 接入 | **1/1** ✅ |
| **Week 6: 发布 + 文档站 + 评估报告** | **4/4** ✅ |

**总完成度估算：~91%**（Week 1-6 全部达成 · model-pull 真活让 onnx 路径用户可达）

### 下一阶段建议（按推荐优先级）

| 候选 | 价值 | 工作量 |
|---|---|---|
| 🟢 **Day 20c: onnx 真活**（BGE-M3 真语义 embedding 端到端） | 解锁真 RAG · 不再是 mock | 2-3 commit |
| 🟢 **用户首推 master + 配 Settings→Pages** | 文档站上线 | 0.5 commit · 用户 UI |
| 🟢 **用户准备 BGE-M3 tarball + 推 models-v1 release** | model-pull `--url` 真活 | 0.5 commit · 用户准备 |
| 🟡 **用户决策 push tag `v1.0.0`** | 1.0 公开 release | 0.5 commit · 用户操作 |
| 🟡 **Day 17: DevEco Plugin MVP** | 关键路径主战场 | 5+ commit |
| 🟡 task #81：升 lancedb 主版本 | 解锁向量库 | 1-2 commit |
| ⚪️ ArkTS custom grammar | ArkTS struct 方法提取 | 大工程 |

**Agent 推荐**：**Day 20c onnx 真活**（让 model-pull 拉的真模型立刻能被 index/query 用 · 真语义检索代替 mock-384 · 解锁产品核心价值）。

### 重要的"非完成"项

- ❌ Day 20c onnx 真活（BGE-M3 真语义 embedding）
- ❌ models-v1 release tarball 准备（用户操作）
- ❌ 用户首次推 master 触发 book.yml + Settings→Pages
- ❌ 用户决策 push tag v1.0.0
- ❌ Day 17 DevEco Plugin MVP
- ❌ task #81 lancedb 主版本升级
- ❌ ArkTS struct method extraction
