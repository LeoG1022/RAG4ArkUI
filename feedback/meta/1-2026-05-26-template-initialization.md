# 1 — template-initialization

> 日期：2026-05-26
> 触发：从 crossplatform-harness 工程抽取元框架层，建立独立可复用模板仓库
> 类型：工程结构调整

---

## 用户提出的要求

将 crossplatform-harness 中非业务相关的配置、skill、脚本等抽离成一个可复用的工程模板，使新项目可以直接克隆并使用完整的 AI agent 协作基础设施。

## Agent 给出的修改建议

分两层拆分：
- **元框架层**（可复用）：CLAUDE.md、AGENTS.md、ONBOARDING.md、scripts/ 框架脚本、.claude/skills/、.claude/references/、feedback/ 归档体系、stats/ 统计体系、Git hooks、跨工具入口文件
- **业务层**（留在原项目）：benchmarks/、kmp-workspace/、arkuix-workspace/、reports/、51 条历史 feedback

关键策略：
1. 5 个 Skill 和全部 references 文件随模板保留（它们是业务变更归档机制的核心载体：`feature_log: true` + `new-feature-log.sh`）
2. CLAUDE.md 删除 3 个业务章节（核心工作流/最优写法/ArkUI-X编译验证），保留全部元框架协议
3. AGENTS.md 删除 benchmarks/kmp-workspace/arkuix-workspace 子目录条目（避免 M-LINK-DEAD FAIL）
4. scripts/check-api-parity.sh 删除 8 条 ArkUI-X 专属规则，保留通用骨架
5. scripts/install-hooks.sh 删除 ArkUI-X smoke-run 步骤 4
6. feedback/DESIGN.md 删除 15 条项目特定决策，保留 2 条通用格式示例

## 多轮互动

用户审阅 Plan 后提问：「在该 Plan 中新模版有归档业务变更的规则吗？」——原 Plan 错误地将 Skills 和 references 归为"业务层不包含"。Agent 更新 Plan，将两者纳入模板（技术理由：skill frontmatter `feature_log_required: true` + `new-feature-log.sh` 调用是业务归档规则的执行载体）。用户审阅更新后的 Plan 并批准。

## 实际改动

- 接口变化：无（新仓库，无已有接口）
- 规则变化：check-api-parity.sh 从 8 条 ArkUI-X 规则变为空骨架（待使用者填写）
- 文件变化：
  - 新建仓库 `/Users/leo/work/agent-harness-template/`
  - 复制（as-is）：RULES_FOR_AGENTS.md、CONVENTIONS.md、.cursorrules、.aiderrules、.clinerules、.windsurfrules、.claude/skills/（5个）、.claude/references/（9个）、.claude/skills/SKILL_SCHEMA.md、.claude/skills/AGENTS.md、.claude/references/AGENTS.md、feedback/AGENTS.md、feedback/refactor-rules.md、stats/AGENTS.md、scripts/（18个框架脚本）
  - 修改：CLAUDE.md（删3业务章节）、AGENTS.md（删3业务目录条目，Rule#16改为注释）、ONBOARDING.md（删历史表）、feedback/DESIGN.md（删15条决策，保留2条示例）、scripts/classify-change.sh（业务路径改为通用占位符）、scripts/check-api-parity.sh（删8条规则，保留骨架）、scripts/install-hooks.sh（删步骤4）、feedback/AGENTS.md（删业务路径条目，删死链）
  - 新建：README.md、.gitignore、.claude/settings.json、.claude/skills/example-skill.md、.claude/references/example-mapping.md、scripts/AGENTS.md、scripts/init-project.sh、reports/AGENTS.md
- 配置变化：.claude/settings.json 新建（通用权限模板，移除 xcrun/hvigor 等项目特定权限）

## 执行生效后总结

### 实际产出

| 检查项 | 结果 |
|---|---|
| `check-consistency.sh` | 无 FAIL；WARN 为预期（空 feedback/M-NO-VERIFY-BAN 初次克隆/M-README-PURE/test fixtures 缺失） |
| M-LINK-DEAD | PASS（已清理所有业务目录死链）|
| M-SKILL-01 | PASS（6 skill 文件 == CLAUDE.md 6 行）|
| M-SKILL-TABLE-SYNC | PASS（regenerate-skill-table 执行后同步）|
| M-SKILL-REF-VALID | PASS（移除 bench-all-ios.sh 引用）|
| `install-hooks.sh` | 4 个 hook 安装成功 |

### 前后对比

| | 原项目 | 模板 |
|---|---|---|
| feedback 历史 | 51 条 | 0 条（本轮第 1 条）|
| check-api-parity 规则 | 8 条（ArkUI-X 专属）| 0 条（占位符）|
| 业务目录 | benchmarks/kmp-workspace/arkuix-workspace | 无 |
| README | 业务用法说明 | 模板克隆引导（5步 Quick Start）|

### 实测验证

```
check-consistency.sh: WARN（exit 2）— 无 FAIL
regenerate-skill-table.sh: ✅ SKILL-TABLE 已同步
install-hooks.sh: 4 个 hook 安装成功
```

### 残留 / 下一轮处理

- [ ] Phase 6：克隆验证（cd /tmp && git clone /Users/leo/work/agent-harness-template test-clone && cd test-clone && bash scripts/init-project.sh && bash scripts/check-consistency.sh && rm -rf /tmp/test-clone）
- [ ] 使用者按项目定制 scripts/check-api-parity.sh 规则（当前为空骨架）
- [ ] 考虑是否需要将本模板推送到远端 Git 仓库
