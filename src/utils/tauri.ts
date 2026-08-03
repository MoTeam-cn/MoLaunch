/**
 * Tauri API 封装工具（统一 re-export 入口，实现按域拆分到 ./api/*）
 *
 * 仅为兼容 `import * as tauri from '@/utils/tauri'` 用法，新函数请按域加入对应子文件。
 */

export * from './api/sdk'
export * from './api/auth'
export * from './api/authlib'
export * from './api/version'
export * from './api/personalization'
export * from './api/java'
export * from './api/loader'
export * from './api/launch'
export * from './api/system'
export * from './api/config'
export * from './api/skin'
export * from './api/image-cache'
export * from './api/developer'
export * from './api/plugins'
