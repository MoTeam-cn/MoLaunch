/* eslint-env node */
require('@rushstack/eslint-patch/modern-module-resolution')

module.exports = {
  root: true,
  extends: [
    'plugin:vue/vue3-recommended',
    'eslint:recommended',
    '@vue/eslint-config-typescript',
    '@vue/eslint-config-prettier/skip-formatting',
  ],
  parserOptions: {
    ecmaVersion: 'latest',
  },
  rules: {
    'vue/multi-word-component-names': 'off',
    // 下划线前缀参数/变量表示"有意忽略"，rest 解构中忽略的兄弟字段不视为未使用
    '@typescript-eslint/no-unused-vars': [
      'warn',
      { argsIgnorePattern: '^_', ignoreRestSiblings: true },
    ],
  },
  ignorePatterns: [
    'src-tauri/resources/view/*.min.js',
    'src-tauri/resources/wasm/*.js',
    // Node 工具脚本（资产生成/CI 上传等），非前端代码，不参与前端 lint
    'scripts/**',
  ],
}
