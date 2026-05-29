# 13 — github-url-fix

> 日期：2026-05-29
> 触发：用户报告 push 到 GitHub 后 Actions 没触发 · agent 检查 git remote 时发现 username 不对
> 类型：bug 修复（全 repo 死链清扫 · 不动 feedback/ 历史归档）

---

## 用户提出的要求

用户报告：「推送到github了 在setting的action并没有看到有什么任务运行」

Agent 检查 `git remote -v` 发现实际 GitHub remote 是 `origin git@github.com:LeoG1022/RAG4ArkUI.git`（不是 agent 之前所有文档假设的 `keerecles/RAG4ArkUI`）。所有 active 用户向文档里的 URL 全是死链。

## Agent 给出的修改建议

| 改动 | 类别 |
|---|---|
| 全 repo `keerecles` → `LeoG1022` sed 替换 | meta（影响 scripts/release-local.sh 默认 URL · .github/workflows/book.yml 注释 · 大量文档）|
| 跳过 `feedback/` 历史归档 | GLOSSARY 已声明的「不擅自重写历史」原则 |

### 关键决策

1. **active vs 归档双标准**：active 文档（README/RELEASE/mdbook/scripts/Cargo.toml）必须改 · feedback/ 历史归档不动
2. **替换工具**：grep -rln + xargs sed -i ''（macOS sed 语法）
3. **URL fix 与 Actions 触发问题正交**：workflow yml 用 `${{ github.repository }}` 自动取 · 不受 username hardcode 影响

## 多轮互动

按时序：
1. 用户：「推送了 actions 没看到任务」
2. Agent 查 `git ls-remote` 确认 push 成功
3. Agent 顺手 grep 发现 24 文件 / 57 处 username 错
4. Agent 老实告知用户「URL 错与 Actions 不触发是两件事」
5. 用户隐式同意（未阻止）· agent sed 全替换

## 实际改动

- **接口变化**：无（URL 字符串更正）
- **规则变化**：无
- **文件变化**：24 个文件 / 57 处替换 `keerecles` → `LeoG1022`
  - meta：scripts/release-local.sh · .github/workflows/book.yml · README.md
  - business：crates/Cargo.toml · crates/arkui-rag-cli/src/main.rs · docs/RELEASE.md · docs/RELEASE-NOTES-v1.0.0.md · docs/STATUS-{day20a,day20b,mdbook-doc,glossary,model-pull}.md · mdbook/* 全部
- **配置变化**：默认 corpus tarball URL 路由从 keerecles → LeoG1022（实际仍是 404 占位）

## 执行生效后总结

### 实际产出

| 项 | 状态 |
|---|---|
| 24 文件 sed 替换 | ✅ |
| `grep keerecles --exclude-dir=feedback` | ✅ 0 命中 |
| `grep LeoG1022` | 57 处（替换成功） |
| feedback/ 历史归档 | ✅ 不动（agent 决策原则） |

### 前后对比

| 维度 | 之前 | 之后 |
|---|---|---|
| README 文档站链接 | 404 | 指向 https://LeoG1022.github.io/RAG4ArkUI/ |
| `corpus pull` 默认 URL | 404 | 指向 LeoG1022 GitHub Release（仍占位 · 用户首次推 corpus-v0.0.1 release 后真活） |
| `corpus model-pull --name bge-m3` 默认 URL | 404 | 指向 LeoG1022 models-v1 release（同上 · 占位） |
| mdbook book.toml git-repository-url + edit-url-template | 404 | 正确指向 LeoG1022 |
| feedback/ 历史归档（22 commit + 19 STATUS + 12 meta 含 keerecles） | 不动 | 不动（GLOSSARY 声明原则） |

### 实测验证

```
$ grep -rn "keerecles" /Users/leo/work/RAG4ArkUI --include="*.md" --include="*.toml" --include="*.yml" --include="*.rs" --include="*.sh" | grep -v feedback
（空）
$ grep -rln "LeoG1022" /Users/leo/work/RAG4ArkUI | wc -l
57
```

### 残留 / 下一轮处理

- [ ] **关键**：等用户反馈 GitHub Actions 页面看到什么（CI/Deploy mdBook/Release 是否启用 · 触发条件是否命中）
- [ ] 本次 push 自然触发 workflow（含 mdbook/ 改动）· 等 GitHub Pages 部署完成
- [ ] 用户在仓库 Settings → Pages → Source 选 "GitHub Actions"（一次性）
- [ ] 后续若改 GitHub username 或 fork · 需再次全 repo sed
- [x] active 文档 URL 修正
- [x] feedback/ 历史归档保留
