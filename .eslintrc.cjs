/* eslint-env node */
module.exports = {
  // 标记这是根配置，ESLint 不会再向上查找
  root: true,
  
  // 解析器选项，让 ESLint 知道我们在用最新的 JS 语法和模块
  parserOptions: {
    ecmaVersion: 'latest',
  },

  // 扩展：继承一系列预设好的规则，这是配置的核心
  extends: [
    'eslint:recommended', // ESLint 官方推荐的基础规则
    'plugin:vue/vue3-recommended', // Vue 3 社区推荐的规则
    '@vue/typescript/recommended', // Vue 社区推荐的 TypeScript 规则
    'prettier', // 使用 prettier 推荐的配置，关闭与 Prettier 冲突的规则
    'plugin:prettier/recommended' // 将 Prettier 作为 ESLint 规则来运行，确保这是最后一个
  ],
  
  // 插件列表
  plugins: [
    // '@typescript-eslint' 已经被 @vue/typescript/recommended 包含
    // 'prettier' 已经被 plugin:prettier/recommended 包含
  ],

  // 自定义规则：可以在这里覆盖或添加你自己的规则
  rules: {
    // 例如，你可以在这里关闭某条你觉得烦人的规则
    // 'vue/multi-word-component-names': 'off',
  },
};