/**
 * tab 选中态 URL 持久化（NavSidebar / SubTabBar 共用）
 *
 * onMounted 从 ?{key}= 恢复（isValid 校验），变化时 router.replace 同步 URL（不产生历史记录）。
 */
import { watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export function useTabPersistence(
  /** 当前选中 tab（getter，响应式） */
  current: () => string,
  /** 校验 tab 是否有效（在分类列表中且未禁用） */
  isValid: (tab: string) => boolean,
  /** tab 变化回调（onMounted 恢复时触发，用于通知父组件更新选中态） */
  onChange: (tab: string) => void,
  /** URL query 键名（默认 'tab'，顶部子菜单用 'subtab' 避免与左侧菜单冲突） */
  key = 'tab',
) {
  const route = useRoute()
  const router = useRouter()

  // 页面加载时从 URL query 恢复选中项（刷新页面保留路径）
  // 跳过无效项：避免恢复到不在分类列表中的 tab
  onMounted(() => {
    const tab = route.query[key] as string | undefined
    if (tab && tab !== current() && isValid(tab)) {
      onChange(tab)
    }
  })

  // 选中项变化时同步到 URL query（不产生历史记录，保留其他 query 参数）
  watch(() => current(), (val) => {
    const existing = route.query[key] as string | undefined
    if (val !== existing) {
      router.replace({ query: { ...route.query, [key]: val } })
    }
  })
}
