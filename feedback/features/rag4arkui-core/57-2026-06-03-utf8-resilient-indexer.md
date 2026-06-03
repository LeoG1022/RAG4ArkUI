# 57 — utf8-resilient-indexer

> 日期：2026-06-03
> 涉及代码：`crates/arkui-rag-indexer/src/lib.rs` · `corpus/official/arkui-x/zh-cn/application-dev/reference/arkui-ts/ts-transition-animation-geometrytransition.md`
> 类型：bug fix（Phase B 3.2h build 死掉的 UTF-8 错根因 + 防御）

## 本轮目标

Round 52 Phase B 跑 application-dev 全量 build（590 files）· **193 分钟（3.2 小时）后死于** `error: io error: stream did not contain valid UTF-8`：

- index.json **没 persist** · 3.2h 算力作废
- BM25 / Tantivy 已 commit 16360 chunks（异步先完成）
- 但 vector store 没保存 · 整轮废

Python scan 1066 .md 真有 **1 个 GBK 编码文件**：
`reference/arkui-ts/ts-transition-animation-geometrytransition.md` · 标题"# 组件内隐式共享元素转场"。
其余 1065 全 UTF-8。

本轮根本解决：
- `indexer/src/lib.rs:80` `read_to_string` 改 lossy fallback · 撞坏字节不死
- 转码罪魁文件 GBK→UTF-8 · 保留正常中文内容

## Plan

### 决策 A · 改 indexer 容错（防御未来）

```rust
let bytes = tokio::fs::read(&path).await?;
let content = match std::str::from_utf8(&bytes) {
    Ok(s) => s.to_string(),
    Err(e) => {
        tracing::warn!(
            "{} 非 UTF-8 (pos {}): ... · 用 from_utf8_lossy 兜底",
            path.display(), e.valid_up_to(), e
        );
        String::from_utf8_lossy(&bytes).into_owned()
    }
};
```

策略：
- 单文件读 + 检测 UTF-8 · 失败时 lossy 替换坏字节为 U+FFFD
- log warn 提示用户修
- 继续 build · 不死

替代方案：
- A · 直接 `read_to_string`（被否：1 个坏文件就 fail · 3h 算力作废）
- B · 检测后 skip 坏文件（被否：少 1 个 chunk · 用户期望整 corpus 覆盖）
- **C · lossy fallback（本次选）**：坏字节替换 + log + 继续

### 决策 B · 转码罪魁文件 GBK → UTF-8

```bash
iconv -f GBK -t UTF-8 ts-transition-animation-geometrytransition.md > tmp
mv tmp ts-transition-animation-geometrytransition.md
```

转码前：`file` 报 `ISO-8859 text` · 内容 `# [BAD]������ʽ����Ԫ`
转码后：`# 组件内隐式共享元素转场` · 中文正确

策略：
- 优先选 GBK iconv（最常见中文遗留编码）
- 修复在 source · 不依赖 lossy 替换（lossy 后 RAG 检索质量会因 U+FFFD 受影响）

为啥不 UTF-8 BOM 重新存？BOM 在 GBK 也可能存在。直接 iconv 一步到位。

### 决策 C · 不动 `corpus/official/openharmony/`

OpenHarmony 109MB / 7343 .md 也可能含 GBK 文件 · 但本轮**不动**：
- OpenHarmony 不入 git（gitignore 屏蔽）· 用户跑 collect-corpus 时随时可重拉
- 改 indexer 容错已足够 · OpenHarmony 撞坏文件 lossy 兜底
- 真要 fix · 应该是 Round 54 candidate · 写 `scripts/check-corpus-utf8.sh` 扫所有 corpus 源

### 不动

- chunker / cli / retrieval 不动
- BGE-M3 模型不动
- Tantivy / lancedb 不动

## 对话摘要

| 轮次 | 用户 | Agent |
|---|---|---|
| 1 | Phase B 跑 3.2h 撞 UTF-8 错 · index.json 没出 | 报告 + 推 4 个 fix 选项 |
| 2 | 「要先解决 index 失败问题」 | python scan 找 1 个 GBK 文件 · 改 indexer lossy + iconv 转码罪魁 |

## 改动要点

### 修改
- `crates/arkui-rag-indexer/src/lib.rs:80-95` · `read_to_string` → `read + from_utf8_lossy` 容错
- `corpus/official/arkui-x/.../ts-transition-animation-geometrytransition.md` · GBK→UTF-8 iconv

### 不动
- index.json 格式
- 既有 cli 接口
- 591 个其他正确 UTF-8 文件

## 验证结果

```bash
# 1. python 扫验证 corpus 全树 UTF-8
python3 -c "..."   # ✓ 0 个非 UTF-8

# 2. cargo build 重新（10-15 分钟）
cargo build --release ...,onnx     # ⏭ 正在跑

# 3. 重启 Phase B build（3.2h · 这次应过）
arkui-rag index --source corpus/official/arkui-x/zh-cn/application-dev \
    --embedder onnx --model-path ~/.arkui-rag/models/bge-m3 \
    --index-path /tmp/rag-full-build-v2/index/index.json \
    --bm25 tantivy
```

## 残留 / 下一轮

- [x] python scan 找到 1 个 GBK 文件
- [x] iconv 转码罪魁 GBK→UTF-8
- [x] indexer lossy fallback 防御
- [ ] **cargo build 完成 + install + 重启 Phase B build**
- [ ] **写 `scripts/check-corpus-utf8.sh`** · 自动扫 corpus/official/ 全树 · CI 跑前先 check（Round 54 候选）
- [ ] **collect-corpus.sh 加 UTF-8 验证步骤** · 拉完自动跑 check + log warn
- [ ] **OpenHarmony 109MB 可能也含 GBK 文件**：下次拉 OH 后跑 check
- [ ] **persist 阶段加 progress** · 让用户看到 vector store 写 index.json 的进度（当前 silent · 死掉无诊断）
- [ ] **build 加 checkpoint**：每 N chunks persist 一次 · 死掉重启从 checkpoint 续
