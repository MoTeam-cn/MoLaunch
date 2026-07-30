/**
 * 侧边栏 tab 选中态 URL 持久化
 *
 * 机制（抽取自 NavSidebar.vue 原内联实现）：
 * - onMounted：从 route.query.tab 读取，若有效且与当前值不同则通过 onChange 通知调用方恢复
 * - watch：当前值变化时 router.replace 写入 URL（不产生历史记录，保留其他 query 参数）
 *
 * # 复用方
 * - NavSidebar.vue：扁平分类 + children 子菜单（isValid 需递归 children）
 * - DownloadSidebar.vue：top + community 两组分类（isValid 检查两组列表）
 *
 * # 为什么抽 composable
 * 两处侧边栏需要相同的 URL 同步逻辑，原 NavSidebar 内联实现无法被 DownloadSidebar 复用。
 * 抽出后 NavSidebar 改用本 composable（逻辑等价），DownloadSidebar 新增同能力，避免重复代码。
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
) {
  const route = useRoute()
  const router = useRouter()

  // 页面加载时从 URL query.tab 恢复选中项（刷新页面保留路径）
  // 跳过无效项：避免恢复到不在分类列表中的 tab
  onMounted(() => {
    const tab = route.query.tab as string | undefined
    if (tab && tab !== current() && isValid(tab)) {
      onChange(tab)
    }
  })

  // 选中项变化时同步到 URL query（不产生历史记录，保留其他 query 参数）
  watch(() => current(), (val) => {
    const existing = route.query.tab as string | undefined
    if (val !== existing) {
      router.replace({ query: { ...route.query, tab: val } })
    }
  })
}
