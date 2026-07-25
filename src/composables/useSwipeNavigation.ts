import { ref, computed, type ComputedRef, type Ref, nextTick } from 'vue'

/**
 * 账号卡片轮播的滑动 / 滚轮导航 composable
 *
 * - 拖动超过阈值（40px）切换上/下一张
 * - 鼠标滚轮左右切换（带 300ms 节流，与切换动画时长匹配）
 * - 暴露 cardTransform 供模板 :style 绑定
 *
 * @param totalCards 总卡片数（含「添加账号」卡片）
 * @param currentIndex 当前索引（外部管理，会被本 composable 通过 onSwitch 间接修改）
 * @param onSwitch 用户触发切换后的回调（通常是父组件的 switchTo）
 */
export function useSwipeNavigation(
  totalCards: ComputedRef<number>,
  currentIndex: Ref<number>,
  onSwitch: (newIndex: number) => void,
) {
  const isDragging = ref(false)
  const dragOffset = ref(0)
  const dragMoved = ref(false)
  /** 松手后回弹动画进行中（期间保留 transition） */
  const isAnimating = ref(false)
  let dragStartX = 0
  let lastWheelTime = 0
  const WHEEL_THROTTLE_MS = 300
  const SWITCH_THRESHOLD = 40

  /** 当前捕获的 pointerId，用于 setPointerCapture/releasePointerCapture */
  let capturedPointerId: number | null = null

  function onPointerDown(e: PointerEvent) {
    // 只响应主键（左键）或触摸
    if (e.button !== 0 && e.pointerType === 'mouse') return
    // 如果 pointerdown 起源于交互元素（按钮/链接/输入框等），跳过拖拽处理，
    // 让 click 事件正常派发到按钮，避免 setPointerCapture 劫持点击导致按钮无响应。
    // 使用 composedPath 而非 target.closest，因为 SVG 子元素（如 <path>）的 closest
    // 在某些 WebView 中可能不跨越 SVG-HTML 边界，导致按钮点击被错误识别为拖拽。
    const path = e.composedPath()
    for (const el of path) {
      if (!(el instanceof Element)) continue
      const tag = el.tagName.toLowerCase()
      if (
        tag === 'button' ||
        tag === 'a' ||
        tag === 'input' ||
        tag === 'select' ||
        tag === 'textarea' ||
        el.getAttribute('role') === 'button'
      ) return
    }
    isDragging.value = true
    isAnimating.value = false
    dragMoved.value = false
    dragStartX = e.clientX
    dragOffset.value = 0
    // 捕获指针：即使移出容器外部也能持续收到 pointermove/pointerup
    capturedPointerId = e.pointerId
    try {
      (e.currentTarget as Element).setPointerCapture(e.pointerId)
    } catch {
      // 某些环境不支持 setPointerCapture，静默忽略
    }
  }

  function onPointerMove(e: PointerEvent) {
    if (!isDragging.value) return
    const dx = e.clientX - dragStartX
    if (Math.abs(dx) > 4) dragMoved.value = true
    // 边界阻尼：拖到第一张左侧或最后一张右侧时，阻尼减半
    const idx = currentIndex.value
    const total = totalCards.value
    if ((idx === 0 && dx > 0) || (idx === total - 1 && dx < 0)) {
      dragOffset.value = dx * 0.35
    } else {
      dragOffset.value = dx
    }
  }

  function onPointerUp(e: PointerEvent) {
    if (!isDragging.value) return
    isDragging.value = false
    isAnimating.value = true

    // 释放指针捕获
    if (capturedPointerId !== null) {
      try {
        (e.currentTarget as Element)?.releasePointerCapture(capturedPointerId)
      } catch {
        // 静默忽略
      }
      capturedPointerId = null
    }

    const shouldSwitch = Math.abs(dragOffset.value) > SWITCH_THRESHOLD
    if (shouldSwitch) {
      if (dragOffset.value < 0 && currentIndex.value < totalCards.value - 1) {
        onSwitch(currentIndex.value + 1)
      } else if (dragOffset.value > 0 && currentIndex.value > 0) {
        onSwitch(currentIndex.value - 1)
      }
    }
    // 清零 dragOffset，让 transition 把卡片平滑带回目标位置
    dragOffset.value = 0

    // 动画结束后取消 isAnimating 标记
    nextTick(() => {
      setTimeout(() => {
        isAnimating.value = false
      }, 320)
    })
  }

  /** 鼠标滚轮左右切换（带节流，防止快速滑动并发请求） */
  function onWheel(e: WheelEvent) {
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
    isAnimating,
    cardTransform,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onWheel,
  }
}
