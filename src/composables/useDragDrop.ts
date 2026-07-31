/**
 * 全局文件拖拽安装 composable（聚合入口）
 *
 * 在 App.vue 根组件调用 `useDragDrop()` 注册 Tauri v2 `onDragDropEvent`，
 * 根据文件扩展名路由到不同处理逻辑：
 *
 * - `.zip` / `.mrpack` → 整合包安装（弹窗输入实例名 → installLocalModpack → installMerged）
 * - `.jar` / `.litemod` / `.disabled` / `.old` → Mod 安装（弹窗选择目标版本 → installMod）
 * - `.rar` → 拒绝并提示用户解压后重新压缩为 zip
 * - 其他 → 提示无法识别
 *
 * 拖拽进入时通过 `dragState` 暴露 enter/over/leave 状态，驱动 DragOverlay 全局遮蔽层。
 *
 * 按职责拆分到 `./useDragDrop/` 子文件，本文件仅做 re-export 以保持
 * `@/composables/useDragDrop` 路径对调用方完全兼容：
 * - `state.ts`：拖拽状态、扩展名常量、路径工具函数、classifyDrag、hideOverlay
 * - `handlers.ts`：文件类型分发与安装处理（handleFileDrop / handleModpackDrop / handleModDrop 等）
 *
 * `useDragDrop()` 生命周期函数因依赖 Vue onMounted/onUnmounted，保留在主文件。
 */

import { onMounted, onUnmounted } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'

// re-export 子模块全部导出，保持调用方路径兼容
export * from './useDragDrop/state'
export * from './useDragDrop/handlers'

// 仅 useDragDrop() 生命周期函数需要的状态/处理函数
import { dragState, classifyDrag, hideOverlay } from './useDragDrop/state'
import { handleFileDrop } from './useDragDrop/handlers'

/**
 * 注册全局拖拽事件监听
 *
 * 必须在 onMounted 中调用，onUnmounted 中取消监听。
 * 返回取消监听函数以便测试或手动卸载。
 *
 * 事件流：
 * - enter：拖拽进入窗口，根据 paths 预判类型，显示 DragOverlay
 * - over：拖拽在窗口内移动，保持 DragOverlay 显示
 * - leave：拖拽离开窗口，隐藏 DragOverlay
 * - drop：释放，隐藏 DragOverlay，分发到对应处理函数
 */
export function useDragDrop(): () => void {
  let unlisten: (() => void) | null = null

  onMounted(async () => {
    try {
      const webview = getCurrentWebview()
      unlisten = await webview.onDragDropEvent(async (event) => {
        const payload = event.payload
        switch (payload.type) {
          case 'enter': {
            const paths = payload.paths ?? []
            const { kind, hint } = classifyDrag(paths)
            dragState.active = true
            dragState.kind = kind
            dragState.hint = hint
            return
          }
          case 'over':
            // 保持遮蔽层显示，无需更新
            return
          case 'leave':
            hideOverlay()
            return
          case 'drop': {
            hideOverlay()
            const paths = payload.paths
            if (!paths || paths.length === 0) return
            await handleFileDrop(paths)
            return
          }
        }
      })
    } catch (err) {
      // onDragDropEvent 在某些平台可能不可用，静默失败
      console.warn('[DragDrop] 注册拖拽事件监听失败:', err)
    }
  })

  onUnmounted(() => {
    unlisten?.()
    unlisten = null
  })

  return () => {
    unlisten?.()
    unlisten = null
  }
}
