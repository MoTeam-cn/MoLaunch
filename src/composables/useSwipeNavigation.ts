import { ref, computed, type ComputedRef, type Ref } from 'vue'

/**
 * 账号卡片轮播的滑动 / 滚轮导航 composable
 *
 * - 拖动超过阈值（60px）切换上/下一张
 * - 鼠标滚轮左右切换（带 300ms 节流，与切换动画时长匹配）
 * - 暴露 cardTransform 供模板 :style 绑定
 *
 * @param totalCards 总卡片数（含「添加账号」卡片）
 * @param currentIndex 当前索引（外部管理，会被本 composable 通过 onSwitch 间接修改）
 * @param onSwitch 用户触发切换时的回调（通常是父组件的 switchTo）
 */
export function useSwipeNavigation(
  totalCards: ComputedRef<number>,
  currentIndex: Ref<number>,
  onSwitch: (newIndex: number) => void,
) {
  const isDragging = ref(false)
  const dragOffset = ref(0)
  const dragMoved = ref(false)
  let dragStartX = 0
  let lastWheelTime = 0
  const WHEEL_THROTTLE_MS = 300

  function onPointerDown(e: PointerEvent) {
    isDragging.value = true
    dragMoved.value = false
    dragStartX = e.clientX
    dragOffset.value = 0
  }
  function onPointerMove(e: PointerEvent) {
    if (!isDragging.value) return
    const dx = e.clientX - dragStartX
    if (Math.abs(dx) > 4) dragMoved.value = true
    dragOffset.value = dx
  }
  function onPointerUp() {
    if (!isDragging.value) return
    isDragging.value = false
    const threshold = 60
    if (dragOffset.value < -threshold && currentIndex.value < totalCards.value - 1) {
      onSwitch(currentIndex.value + 1)
    } else if (dragOffset.value > threshold && currentIndex.value > 0) {
      onSwitch(currentIndex.value - 1)
    }
    dragOffset.value = 0
  }
  /** 鼠标滚轮左右切换（带节流，防止快速滑动并发请求） */
  function onWheel(e: WheelEvent) {
    // 只在非拖动时响应滚轮（switching 由 onSwitch 回调内部的 switchTo 自行检查）
    if (isDragging.value) return
    const now = Date.now()
    if (now - lastWheelTime < WHEEL_THROTTLE_MS) return

    let direction = 0
    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      direction = e.deltaY > 0 ? 1 : -1
    } else if (e.deltaX !== 0) {
      direction = e.deltaX > 0 ? 1 : -1
    }
    if (direction === 0) return
    e.preventDefault()

    const newIndex = currentIndex.value + direction
    if (newIndex < 0 || newIndex >= totalCards.value) return
    lastWheelTime = now
    onSwitch(newIndex)
  }

  /** 卡片滑动 transform */
  const cardTransform = computed(() => {
    if (isDragging.value && dragMoved.value) {
      return `translateX(calc(-${currentIndex.value * 100}% + ${dragOffset.value}px))`
    }
    return `translateX(-${currentIndex.value * 100}%)`
  })

  return {
    isDragging,
    dragMoved,
    cardTransform,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onWheel,
  }
}
