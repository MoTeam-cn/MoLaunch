## 变更说明

请简洁说明本次改动的内容与动机，遵循仓库提交规范（见 `DEVELOPMENT_GUIDELINES.md` 第二节）：

- 变更类型 + 影响范围：`type(scope): 描述`
- 提交信息需带 `!c` 标记（跳过 CI 重复构建）
- 提交信息使用泛化技术描述，禁止第三方项目代号

```text
例如：fix(launch): 修复 1.21 加载器兼容性问题 !c
```

## 改动清单

- [ ] 描述本次改动的核心点（可选，列表形式）
- [ ] 已同步更新 `CHANGELOG.md`
- [ ] 相关验证通过：`cargo check` / `npm run typecheck` / `npm run lint`