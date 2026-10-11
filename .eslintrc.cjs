/* eslint-env node */
require('@rushstack/eslint-patch/modern-module-resolution')

module.exports = {
  root: true,
  extends: [
    'plugin:vue/recommended',
    'eslint:recommended',
    // 原 @vue/eslint-config-typescript@13 只做三件事：指定 vue-eslint-parser、
    // 把 @typescript-eslint/parser 作为内部 parser、启用 TS 推荐规则。
    // 这里直接展开，以便升级到 @typescript-eslint v8（v8 的 typescript-estree
    // 已改用 tinyglobby，不再引入 globby → fast-glob → micromatch → braces 链条）。
    'plugin:@typescript-eslint/recommended',
    '@vue/eslint-config-prettier/skip-formatting',
  ],
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    ecmaVersion: 'latest',
    sourceType: 'module',
  },
  env: {
    browser: true,
    es2022: true,
  },
  overrides: [
    {
      // TS 的类型检查（vue-tsc）已覆盖未定义标识符，typescript-eslint 官方建议关闭 no-undef；
      // 原 @vue/eslint-config-typescript 通过 override 对 .vue 生效，这里显式补回。
      files: ['*.ts', '*.tsx', '*.mts', '*.cts', '*.vue'],
      rules: {
        'no-undef': 'off',
      },
    },
    {
      // Node 侧配置文件使用 require()，不属于前端模块
      files: ['*.cjs'],
      rules: {
        '@typescript-eslint/no-require-imports': 'off',
      },
    },
  ],
  rules: {
    'vue/multi-word-component-names': 'off',
    // 下划线前缀参数/变量表示"有意忽略"，rest 解构中忽略的兄弟字段不视为未使用
    '@typescript-eslint/no-unused-vars': [
      'warn',
      { argsIgnorePattern: '^_', ignoreRestSiblings: true },
    ],
    // 以下三条在 @typescript-eslint v7（原 @vue/eslint-config-typescript@13 使用）中不构成
    // 阻塞错误，v8 的 recommended 将其提升为 error。此处保持原有严重度（warn），
    // 避免把 25 处既有代码问题混进本次依赖升级；建议后续单独清理后改回 error。
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-unused-expressions': 'warn',
    '@typescript-eslint/no-empty-object-type': 'warn',
  },
  ignorePatterns: [
    'src-tauri/resources/view/*.min.js',
    // cubiomes 编译产物（emcc 生成），非手写代码，不参与前端 lint
    'src/assets/seedmap/**',
    // Node 工具脚本（资产生成/CI 上传等），非前端代码，不参与前端 lint
    'scripts/**',
  ],
}
