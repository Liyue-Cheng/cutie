/**
 * Task Store - 重构后的入口文件
 *
 * 这个文件现在只是重新导出新的模块化 Task Store
 * 保持向后兼容性，所有现有的导入都能正常工作
 */

// 重新导出新的模块化 Task Store
export { useTaskStore } from '@/stores/task/index'
export * from '@/stores/task/types'
