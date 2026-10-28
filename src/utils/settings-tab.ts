/**
 * 设置页分类的查询参数解析
 *
 * 设置页的分类由左侧菜单自己的 v-model 驱动，外部页面想“直接打开某一页”时
 * 只能靠 URL 上的 `?tab=<分类 id>`。这里把「查询参数 → 合法分类 id」这一步抽成纯函数，
 * 便于单测，也避免各调用点各写一份判断。
 */

/**
 * 从 `route.query.tab` 的值里取出分类 id
 *
 * 只接受字符串并且在 `allowed` 里存在，其余一律返回 null：
 * - 重复参数（`?tab=a&tab=b`）在 vue-router 里是数组，语义不明，直接忽略；
 * - 拼错或不存在的分类不能让页面白屏，保持当前分类即可。
 */
export function resolveSettingsTab(raw: unknown, allowed: readonly string[]): string | null {
  if (typeof raw !== 'string') return null
  const id = raw.trim()
  if (!id) return null
  return allowed.includes(id) ? id : null
}
