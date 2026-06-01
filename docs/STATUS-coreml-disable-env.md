# STATUS — coreml-disable-env

> 配套 feature log：`feedback/features/rag4arkui-core/50-2026-06-01-coreml-disable-env.md`
> 日期：2026-06-01

---

## 当前状态

Round 49 PoC 暴露 ort rc.12 + CoreML EP + BGE-M3 external data 加载 bug。本轮用 env 绕过：cmd_index 进程禁用 CoreML EP · cmd_query 保留 · 同 binary 两端共用。

本阶段交付：
- onnx.rs / reranker_onnx.rs 加 `ARKUI_RAG_DISABLE_COREML` env 检测
- cli cmd_index 入口自动 set env
- 实测：index 真重 build OK · query 仍 ~200ms CoreML 加速
- corpus + index tarball 用真 build 重打（corpus 14KB · index 775KB）

意义：Round 49 PoC 阻塞解除 · Round 49.5 第 1 半完成。等用户给 ArkUI-X / OpenHarmony 真仓库 URL · Round 49.5 第 2 半（收集 + build 真 corpus + index）可立即开做。

## 输入契约

新增 env 变量：

```bash
ARKUI_RAG_DISABLE_COREML=1   # 禁用 CoreMLExecutionProvider 注册
                              # cmd_index 内部自动 set · 用户不感知
                              # 用户可手动 set · 强制走 CPU EP（debug 用）
```

不变项：CLI 接口 / 模型文件 / 索引格式 / 默认 features 全不变。

## 输出契约

| 命令 | CoreML | latency |
|---|---|---|
| `arkui-rag index --embedder onnx` | ❌ 禁用 | ~90s for 8 文件 / 68 chunks（CPU only）|
| `arkui-rag query --embedder onnx` | ✅ 启用 | ~200ms（Round 47 21× 加速保留）|
| `arkui-rag serve --mcp --embedder onnx` | ✅ 启用 | 同上 |

破坏性变更：无（query / serve 路径不变 · 仅 index 阶段降级到 CPU）。

## 验证手段

### Agent 本轮已做

```bash
cargo build --release --features ...,onnx    # 26.46s ✓
arkui-rag index --source corpus/official ... # ✓ 8 files / 68 chunks / 89s
ls -lh /tmp/dist-corpus-v1.0.0/              # corpus 14KB · index 775KB
```

### 用户验证（Round 49.5 第 2 半之前不必）

无需立即跑 · 等收完真 ArkUI-X / OpenHarmony 文档后再做端到端。

## 与上一阶段的关联性

| Round | 主题 | 关系 |
|---|---|---|
| 47 | v1.0.0 + CoreML 21× | 本轮配套 · query 仍 CoreML |
| 49 | corpus 分发 PoC + 暴露 CoreML bug | **本轮修这个 bug** |
| **50（本轮）** | env 绕过 CoreML+external data bug | 解锁 Round 49.5 第 2 半 |

## 完成度 / 下一阶段

### 达成度

| 项 | 状态 |
|---|---|
| onnx.rs / reranker_onnx.rs env 检测 | ✅ |
| cli cmd_index 自动 set env | ✅ |
| build + cp + codesign 新 binary | ✅ |
| index 真重 build 验证 | ✅ |
| corpus + index 重打 | ✅ |
| 双轨归档 + STATUS | ✅ |

### 下一阶段建议

立即（用户做）：
- 提供 ArkUI-X / OpenHarmony 真仓库 URL（GitHub / gitcode）
- 确认 Apache 2.0 重分发 OK

之后（agent 接手 Round 49.5 第 2 半）：
- 写 `scripts/collect-corpus.sh`
- 拉 ArkUI-X + OpenHarmony docs
- build 真 index（CoreML bypass 已就位 · 直接跑）
- 打包 + 推 GitHub Release `corpus-v1.0.0`
- 验证默认 corpus pull URL 真活

长期：
- 跟踪 ort 上游 CoreML + external data bug · 修了后可去掉 env 检测
- 考虑给用户加 cli flag `--disable-coreml` · 比 env 更明示
