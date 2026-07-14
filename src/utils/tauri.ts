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
 * - system.ts       系统操作 + 全局配置（下载/内存/线程/隔离/代理/进度）
 * - skin.ts         皮肤与披风管理
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
export * from './api/skin'
