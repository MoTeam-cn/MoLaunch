/**
 * 客户端自动更新工具（module-level 单例，对外统一入口）
 *
 * 状态与类型见 updater/state，检查/自动轮询见 updater/check，下载安装见 updater/install。
 * 配套组件：`src/components/about/UpdateDialog.vue`
 */
export type { UpdateState, UpdateStatus } from './updater/state'
export { updateState } from './updater/state'
export { checkForUpdate, initAutoCheck } from './updater/check'
export { applyPendingUpdate, closeDialog, downloadAndInstall } from './updater/install'