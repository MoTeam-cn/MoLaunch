/**
 * Tauri API 封装工具（统一 re-export 入口）
 *
 * 实现按域拆分到 `./api/*`：
 * - sdk.ts          SDK 平台信息
 * - auth.ts         离线登录 + 微软登录 + 账号管理
 * - version.ts      版本列表与文件夹管理
 * - personalization.ts  版本个性化 + Mod + 文件补全 + 脚本导出
 * - java.ts         Java 检测/校验/下载
 * - loader.ts       加载器版本查询与合并安装
 * - launch.ts       启动游戏
 * - system.ts       系统操作 + 下载进度查询
 * - config.ts       全局配置读写（getConfig/applyConfig/refreshConfig + 缓存）
 * - skin.ts         皮肤与披风管理
 * - developer.ts    开发者模式（解锁/开关/日志/缓存/存储/系统信息）
 *
 * 此文件仅为兼容现有 `import * as tauri from '@/utils/tauri'` 用法，
 * 不应再在此处直接添加新函数——请按域归类到对应子文件。
 */

export * from './api/sdk'
export * from './api/auth'
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
