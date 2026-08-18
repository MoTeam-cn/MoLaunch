/**
 * 全局文件拖拽安装 composable（聚合入口，App.vue 挂载）
 *
 * 注册 Tauri onDragDropEvent 并按扩展名路由：zip/mrpack→整合包、jar 等→Mod、
 * rar→拒绝解压提示、其他→无法识别；dragState 驱动全局 DragOverlay。
 * 逻辑拆分至 useDragDrop/ 子文件，本文件 re-export 并保留生命周期函数。
 */

import { onMounted, onUnmounted } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'

// re-export 子模块全部导出，保持调用方路径兼容
export * from './useDragDrop/state'
export * from './useDragDrop/handlers'

// 仅 useDragDrop() 生命周期函数需要的状态/处理函数
import { dragState, classifyDrag, getDragSuppressed, hideOverlay } from './useDragDrop/state'
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
        // 页面置位抑制标志时（如模组翻译页局部拖放框），全局拖拽整体静默
        if (getDragSuppressed()) return
        const payload = event.payload
        switch (payload.type) {
          case 'enter': {
            const paths = payload.paths ?? []
            const { kind, hint, status } = classifyDrag(paths)
            dragState.active = true
            dragState.kind = kind
            dragState.hint = hint
            dragState.status = status
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
