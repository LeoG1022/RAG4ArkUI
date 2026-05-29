# 11 — mdbook-deploy-workflow

> 日期：2026-05-29
> 触发：用户指令「按推荐顺序继续」（执行 Day 22 mdBook + 1.0 release notes）
> 类型：工程结构调整（新增 mdbook/ 目录 + CI 部署 workflow + Makefile 工具链）

---

## 用户提出的要求

按 agent 推荐顺序，Day 22 mdBook 文档站 + 1.0 release page：让用户从一个 URL 入口找到所有文档，CI 自动部署到 GitHub Pages。

## Agent 给出的修改建议

| 改动 | 类别 | 内容 |
|---|---|---|
| `mdbook/` 目录（17 个新文件） | business（用户向文档） | book.toml + SUMMARY.md + 13 src/*.md（多数 `{{#include}}` docs/） |
| `.github/workflows/book.yml` | meta（CI workflow yml） | push master 触发 mdbook build + actions/deploy-pages |
| `Makefile` | meta（脚本入口） | + 4 targets: install-mdbook / book-build / book-serve / book-clean |
| `.gitignore` | meta（构建产物排除） | + `/mdbook/book/` |
| `docs/ROADMAP.md` | business（用户文档） | `<slug>` 反引号转义（mdBook HTML parser 兼容） |
| `docs/RELEASE-NOTES-v1.0.0.md` | business（用户文档） | 1.0 release notes 草稿 |
| `README.md` | meta（项目入口） | 顶部加文档站 URL |

### 关键决策

1. **mdBook（而非 docusaurus / vitepress）**：Rust 生态契合 · 部署最简单 · brew/cargo install 即用
2. **`{{#include}}` 内联策略**：单一信任源在 `docs/` · mdbook/src 只做导航与组织
3. **GitHub Pages 用 `actions/deploy-pages@v4`**：官方推荐方案 · 替代旧的 push gh-pages 分支模式
4. **release-notes-v1.0.0 不擅自推 tag**：版本号决策 + push tag 是用户操作

## 多轮互动

无 —— 用户给出「按推荐顺序继续」指令后 agent 直接执行。

构建调试 3 次（agent 自己修，未涉及用户）：
- book.toml 的 `multilingual` 是过时字段
- `git-repository-icon = "fa-github"` 在新版 mdBook 报 Font Awesome 缺图标
- ROADMAP.md 的 `<slug>` 字面值被 mdBook parser 当 HTML 标签

## 实际改动

- **接口变化**：无（Rust 代码 / CLI 不动）
- **规则变化**：无
- **文件变化**：
  - 新增：`mdbook/` 目录 17 个文件
  - 新增：`.github/workflows/book.yml`
  - 新增：`docs/RELEASE-NOTES-v1.0.0.md`
  - 新增：`docs/STATUS-mdbook-doc.md`
  - 修：`Makefile`（+ 4 targets · help 行）
  - 修：`.gitignore`（+ 1 行）
  - 修：`README.md`（顶部加文档站 URL）
  - 修：`docs/ROADMAP.md`（`<slug>` 反引号转义）
- **配置变化**：build artifact `mdbook/book/` 入 .gitignore

## 执行生效后总结

### 实际产出

| 产物 | 状态 |
|---|---|
| `mdbook build` 本地构建 | ✅ 2.8 MB 静态站 · 0 error 0 warning |
| `make book-build` 一键封装 | ✅ |
| `make book-serve` 本地预览 | ⏳ 用户手动验 |
| `.github/workflows/book.yml` YAML 合法 | ✅（ruby 解析 2 jobs） |
| 推 master 首次部署 | ⏳ 用户首推后验 |
| 仓库 Settings → Pages → "GitHub Actions" | ⏳ 用户一次性 |
| `docs/RELEASE-NOTES-v1.0.0.md` 草稿 | ✅ |

### 前后对比

| 维度 | Day 21 完成时 | 本轮（Day 22）完成时 |
|---|---|---|
| 公开文档站 | ❌ 仅 GitHub repo 上散落的 .md | ✅ **GitHub Pages 自动部署 + 全文搜索 + 主题** |
| 文档单一信任源 | docs/ 是源 · 别处 inline 重复 | ✅ docs/ 是源 · mdbook/src 用 `{{#include}}` |
| Makefile book target | 无 | ✅ 4 个（install-mdbook / book-build / book-serve / book-clean） |
| 1.0 release 准备 | 无 | ✅ release-notes-v1.0.0 草稿 |
| Week 6 进度 | 4/4 | **完整收尾 · 总完成度 90%** |

### 实测验证

```
$ mdbook --version
mdbook v0.5.3

$ mdbook build
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `/Users/leo/work/RAG4ArkUI/mdbook/book`

$ du -sh mdbook/book/
2.8M   mdbook/book/

$ ruby -ryaml -e "puts YAML.load_file('.github/workflows/book.yml')['jobs'].keys.inspect"
["build", "deploy"]
```

### 残留 / 下一轮处理

- [ ] **关键**：用户在仓库 Settings → Pages 选 "GitHub Actions"
- [ ] **关键**：用户首次推 master 触发 book.yml 自动部署
- [ ] **关键**：用户决策 push tag `v1.0.0` 时机
- [ ] mdBook 站点 logo / 图标（细节优化）
- [ ] search 权重优化（boost 关键 page）
- [ ] 跟踪 Settings → Pages 配置变化（GitHub UI 改版）
- [x] mdbook book.toml + SUMMARY.md + 13 src/*.md
- [x] `{{#include}}` 单一信任源
- [x] `.github/workflows/book.yml`
- [x] Makefile 4 targets
- [x] `.gitignore` 排除 mdbook/book/
- [x] docs/RELEASE-NOTES-v1.0.0.md 草稿
- [x] README + ROADMAP 同步
- [x] 本地构建通过（2.8 MB · 0 error 0 warning）
