# reports/ — 自动生成产物

此目录存放 skill 或脚本自动生成的报告输出。

---

## 约定

- 报告文件由 skill 脚本**自动生成**，禁止手动编辑
- 文件命名建议：`YYYY-MM-DD_HH-MM/<report-name>.md`（按时间组织，便于追溯）
- 报告属于**业务产物**（`classify-change.sh` 分类为 `business`），不需要关联 feedback

---

## 使用者说明

使用者按项目实际情况填写本目录的具体结构，例如：

```
reports/
├── YYYY-MM-DD_HH-MM/
│   └── comparison.md   ← 某次运行的对比报告
└── ...
```

---

## 下一步

- 想生成报告 → 触发对应 skill（见 `CLAUDE.md` Skill 速查）
- 想了解报告结构 → 读对应 skill 的 frontmatter `description` 字段
