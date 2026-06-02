# 26 — corpus-build-ci

> 日期：2026-06-02
> 触发：Round 49.8 完成（`arkui-rag corpus index-pull` 真活）后 · 默认 URL 路径需要 maintainer CI 推 release 才闭环
> 类型：CI 工作流（新增 .github/workflows/corpus-build.yml + scripts/release-corpus.sh）

---

## 用户提出的要求

> 「go」（接 Round 49.8 收尾时 agent 推荐的下一步 #97 Round 49.6 corpus-build CI）

潜在需求：
1. `arkui-rag corpus index-pull` 现在能拉 · 但默认 URL `corpus-v1.0.0/arkui-rag-index-bge-m3-v1.0.0.tar.gz` 还没真 release
2. maintainer 不应自己手动跑 3.2h build + 手动 gh release · 应该 CI 一键
3. 之后版本升级（v1.1.0 / 加 OpenHarmony 等）也要可重复

## Agent 给出的修改建议

新增两件东西：

**A. `scripts/release-corpus.sh`** —— 把 tar 打包逻辑抽出来 · 本地 maintainer 也能跑
- 输入：--version + --embedder + --index-dir + --corpus-dir + --output-dir
- 输出：corpus tarball + index tarball + SHA256SUMS
- GNU tar 自动启 reproducible options（hash 跨机器稳定）· BSD tar 跳过
- 既给本地 maintainer 备份用 · 也给 CI workflow 调

**B. `.github/workflows/corpus-build.yml`** —— GitHub Actions
- 触发：仅 `workflow_dispatch`（避免误推 3.2h build）
- inputs：version / embedder / langs（默认 `zh-cn`）/ dry_run
- runner：ubuntu-22.04（Linux x86_64 · CPU only · 跟用户本地 macOS BGE-M3 跳 CoreML 等效）
- 步骤：cache cargo + BGE-M3 → collect-corpus → build cli → model-pull (cache miss) → index → release-corpus → upload artifact + push release
- timeout-minutes: 350（5h50m · 留余量）

### 替代方案

- A · push tag corpus-v* 自动触发（被否：误推风险 · 3.2h build 浪费 CI）
- B · 每周 cron 自动 build（被否：ArkUI-X 文档更新频率低 · cron 浪费）
- **C · workflow_dispatch 手动 + cache（本次选）**：maintainer 显式 trigger · 重复 cache 加速

### 关键决策

| 决策 | 选择 | 理由 |
|---|---|---|
| runner OS | ubuntu-22.04 | 与 release.yml 既有 Linux runner 对齐 · CPU only 与本地 macOS auto-detect 等效 |
| 缓存策略 | cargo + BGE-M3 模型 | cargo 提速 build · 模型 2.3GB cache 避免每次重 pull |
| 默认 langs | zh-cn | 1.5h vs zh-cn+en 3.2h · 首次 v1.0.0 选短的 · v1.1.0 可改双语 |
| timeout | 350 分 | 全量 + cache miss 模型下载 + 上传 · 留余量 |
| dry_run | 默认 false | 推 release；测试 workflow 时手动 set true 只上传 artifact |
| concurrency | cancel-in-progress: false | 3.2h build 不允许中途取消 |
| reproducible tar | GNU tar 自动启 | hash 跨机器稳定 · 与 macOS 本地 BSD tar 可对比 |

## 多轮互动

无 —— 用户「go」后 agent 自主设计 CI workflow + release 脚本架构。

## 实际改动

- 接口变化：
  - 新增 maintainer 命令：`bash scripts/release-corpus.sh --version vX.Y.Z ...`
  - 新增 CI workflow：`gh workflow run "Corpus Build" -f version=v1.0.0` 或 GitHub UI 手动 trigger
- 规则变化：无（不动 pre-commit / hook / classify）
- 文件变化：
  - 新增 `scripts/release-corpus.sh`（~120 行 · 参数解析 + tar reproducibility + SHA256SUMS）
  - 新增 `.github/workflows/corpus-build.yml`（~130 行 · workflow_dispatch + 11 步骤）
- 配置变化：
  - workflow input 默认值：`version=v1.0.0 · embedder=bge-m3 · langs=zh-cn · dry_run=false`

## 执行生效后总结

### 实际产出

| 项 | 内容 |
|---|---|
| release-corpus.sh | 本地 + CI 共用 · 跑 Round 49.8 解压目录 + corpus/official → 2.8M + 1.2M + SHA256SUMS ✓ |
| corpus-build.yml | YAML 语法 ruby validator OK · 11 步骤 · timeout 350 分 |
| 用户拉法 | `arkui-rag corpus model-pull --name bge-m3 && arkui-rag corpus index-pull` 闭环 |

### 前后对比

| 操作 | Round 49 PoC | Round 49.5/51（本地）| Round 49.6（本轮）|
|---|---|---|---|
| corpus build | 手动 cli 跑 | 本机 macOS 3 分钟（quick-start）| **CI Linux 全量 3.2h** |
| 打 tarball | 手动 tar 命令 30 行 | 手动 tar 一行 | **scripts/release-corpus.sh** |
| 推 release | 手动 `gh release create` | 不推（PoC）| **workflow 自动推** |
| SHA256SUMS | 没 | 没 | **自动产 + 入 release** |
| 重复 build（v1.1.0）| 翻 round 历史 | 翻 round 历史 | **workflow_dispatch 一行** |

### 实测验证

```bash
# 本地脚本验证（用 Round 49.8 解压目录当假 build 产物）
bash scripts/release-corpus.sh \
    --version v1.0.0 --embedder bge-m3 \
    --index-dir /Users/leo/tmp-index-pull2 \
    --corpus-dir corpus/official \
    --output-dir /tmp/release-test
# 🔧 BSD tar 检测到（macOS）· reproducible options 跳过
# ═══ 打包 corpus ═══  size: 2.8M
# ═══ 打包 index ═══   size: 1.2M
# ═══ SHA256SUMS ═══
# 47adf4c7d94a989f43683f9b8a8030cf346bf142e0c6934800b48334cea87235  arkui-rag-corpus-v1.0.0.tar.gz
# fa723d6de4fe0da64847990bf5825d7f7d9cd2459aee13d70210c3d56f86205e  arkui-rag-index-bge-m3-v1.0.0.tar.gz

# YAML 语法
ruby -ryaml -e "YAML.load_file('.github/workflows/corpus-build.yml')"   # ✓
```

CI 真活验证：用户首次 `gh workflow run` 或 GitHub UI 触发（任务 #97 收尾）。

### 残留 / 下一轮处理

- [x] scripts/release-corpus.sh 抽离 · 本地 + CI 共用
- [x] .github/workflows/corpus-build.yml workflow_dispatch
- [x] cache cargo + BGE-M3 (2.3GB)
- [x] GNU tar reproducible options (CI hash 稳定)
- [x] dry_run flag 支持 artifact-only 测试
- [ ] **用户实操**：触发首次 `Corpus Build` workflow · dry_run=true 先验 workflow 真活 · 再 dry_run=false 推 v1.0.0 release
- [ ] **Round 49.6.1（CI 真跑暴露的问题）**：可能命中 ubuntu runner 内存（7GB）/ 磁盘（14GB）限制 · BGE-M3 模型 2.3GB + corpus + cargo target · 紧（约 11GB 占用）· 若 OOM 需考虑用 large runner 或拆 build
- [ ] **Round 49.6.2**：CI 真活推第一个 corpus-v1.0.0 后 · 跑 `arkui-rag corpus index-pull` 不带 `--from-file` 测默认 URL 真活
- [ ] **Round 49.7**：OpenHarmony 加入 corpus 后 · 加 v1.1.0 build matrix
- [ ] **长期**：mirror 切换（gitee / 阿里云 OSS / Cloudflare R2）应对国内访问
