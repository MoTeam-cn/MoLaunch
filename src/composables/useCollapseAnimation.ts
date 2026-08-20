/**
 * 折叠动画统一工具：生成内容区（grid-template-rows 0fr→1fr）与图标旋转的过渡 class。
 * 内部状态：useCollapseAnimation()；v-for 等外部状态场景用 contentClassOf/iconClassOf 纯函数。
 */
import { computed, ref, type ComputedRef } from 'vue'

export interface CollapseAnimationOptions {
  /** 内容区容器过渡 class（默认 'transition-all duration-300 ease-in-out'） */
  contentTransition?: string
  /** 图标过渡 class（默认 'transition-transform duration-300 ease-in-out'） */
  iconTransition?: string
  /** 展开时图标旋转 class（默认 'rotate-180'） */
  rotateClass?: string
  /** 折叠时图标旋转 class（默认 ''） */
  collapsedRotateClass?: string
  /** 展开时内容区附加 class（如 'opacity-100'） */
  expandedExtra?: string
  /** 折叠时内容区附加 class（如 'opacity-0'） */
  collapsedExtra?: string
}

export function useCollapseAnimation(options: CollapseAnimationOptions = {}) {
  const {
    contentTransition = 'transition-all duration-300 ease-in-out',
    iconTransition = 'transition-transform duration-300 ease-in-out',
    rotateClass = 'rotate-180',
    collapsedRotateClass = '',
    expandedExtra = '',
    collapsedExtra = '',
  } = options

  const isOpen = ref(false)
  const toggle = () => { isOpen.value = !isOpen.value }

  const contentClassOf = (open: boolean) =>
    ['grid', contentTransition, open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]', open ? expandedExtra : collapsedExtra]
      .filter(Boolean)
      .join(' ')
  const iconClassOf = (open: boolean) =>
    [iconTransition, open ? rotateClass : collapsedRotateClass].filter(Boolean).join(' ')

  const contentClass: ComputedRef<string> = computed(() => contentClassOf(isOpen.value))
  const iconClass: ComputedRef<string> = computed(() => iconClassOf(isOpen.value))

  return { isOpen, toggle, contentClass, iconClass, contentClassOf, iconClassOf }
}