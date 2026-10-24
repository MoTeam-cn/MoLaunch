/**
 * 下载管理页「是否该退回上一页」的判定（纯函数，便于单测）
 *
 * 背景：整合安装 = MC 本体下载 + 加载器安装（Fabric/Forge）+ 后处理，
 * 本体下载批次收尾时后端会短暂广播「面板隐藏」。前端若据此直接退回上一页，
 * 就会出现「页面退出了，但游戏还在后台装」的错觉。
 * 因此前端状态熄灭之后必须再问一次后端：只要后端仍在跑，就留在页面上恢复进度。
 */

/** 空闲时的处置：留在页面 / 退回上一页 */
export type IdleAction = 'stay' | 'back'

/**
 * 判定空闲时的处置
 *
 * @param backendActive 后端 isDownloading() 结果：true 仍在跑，false 已结束，undefined 查询失败
 * @param storeActive 前端 store 是否认为正在下载
 */
export function decideIdleAction(params: {
  backendActive: boolean | undefined
  storeActive: boolean
}): IdleAction {
  if (params.storeActive) return 'stay'
  // 查询失败时保守留在页面：宁可多留一屏，也不要把仍在跑的任务当成已结束
  if (params.backendActive !== false) return 'stay'
  return 'back'
}
