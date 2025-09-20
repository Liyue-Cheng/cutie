// /* eslint-env node */
// module.exports = {
//   // 标记这是根配置，ESLint 不会再向上查找
//   root: true,
  
//   // 解析器选项，让 ESLint 知道我们在用最新的 JS 语法和模块
//   parserOptions: {
//     ecmaVersion: 'latest',
//   },

//   // 扩展：继承一系列预设好的规则，这是配置的核心
//   extends: [
//     'eslint:recommended', // ESLint 官方推荐的基础规则
//     'plugin:vue/vue3-recommended', // Vue 3 社区推荐的规则
//     '@vue/typescript/recommended', // Vue 社区推荐的 TypeScript 规则
//     'prettier' // 使用 prettier 推荐的配置，关闭与 Prettier 冲突的规则
//   ],
  
//   // 插件列表
//   plugins: [
//     // '@typescript-eslint' 已经被 @vue/typescript/recommended 包含
//     // 'prettier' 已经被 plugin:prettier/recommended 包含
//   ],

//   // 自定义规则：可以在这里覆盖或添加你自己的规则
//   rules: {
//     // 例如，你可以在这里关闭某条你觉得烦人的规则
//     // 'vue/multi-word-component-names': 'off',
//   },
// };



/* eslint-env node */
module.exports = {
  root: true,
  
  // 指定 ESLint 的解析器，让它能理解 TypeScript
  parser: 'vue-eslint-parser', 
  
  parserOptions: {
    // 为 <script> 部分指定解析器
    parser: '@typescript-eslint/parser',
    ecmaVersion: 'latest',
    sourceType: 'module',
  },

  // 继承的规则集
  extends: [
    'eslint:recommended',
    'plugin:vue/vue3-recommended',
    '@vue/typescript/recommended',
    
    // 【重要】确保这一项在最后！它会关闭所有与 Prettier 冲突的规则。
    'prettier' 
  ],
  
  // 插件列表 (现在为空，因为 extends 已经帮我们做了)
  plugins: [],

  // 自定义规则
  rules: {
    // 你可以在这里覆盖规则，比如关闭组件名必须为多词的规则
    'vue/multi-word-component-names': 'off',
    
    // 如果你发现某些规则与你的习惯冲突，可以在这里关闭它们
    // 例如，关闭对未使用变量的报错（不推荐，但可以）
    // '@typescript-eslint/no-unused-vars': 'off',
  },
};