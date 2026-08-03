/**
 * 搜索历史 composable：localStorage 持久化（molaunch-search-history）
 *
 * 保留最近 5 条、去重、最新在前；读写复用 safeCallSync 范式。
 */
import { ref } from 'vue'
import { safeCallSync } from '@/utils/async'

const STORAGE_KEY = 'molaunch-search-history'
const MAX_HISTORY = 5

export function useSearchHistory() {
  const history = ref<string[]>([])

  function load() {
    safeCallSync(() => {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) {
        const parsed = JSON.parse(saved)
        if (Array.isArray(parsed)) {
          history.value = parsed
            .filter((s): s is string => typeof s === 'string')
            .slice(0, MAX_HISTORY)
        }
      }
    }, 'load search history')
  }

  function persist() {
    safeCallSync(() => {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value))
    }, 'save search history')
  }

  /** 添加搜索词：去重后置顶，保留最近 MAX_HISTORY 条 */
  function add(term: string) {
    const trimmed = term.trim()
    if (!trimmed) return
    const filtered = history.value.filter(h => h !== trimmed)
    filtered.unshift(trimmed)
    history.value = filtered.slice(0, MAX_HISTORY)
    persist()
  }

  /** 删除指定搜索词 */
  function remove(term: string) {
    history.value = history.value.filter(h => h !== term)
    persist()
  }

  /** 清空历史 */
  function clear() {
    history.value = []
    persist()
  }

  load()

  return { history, add, remove, clear }
}
