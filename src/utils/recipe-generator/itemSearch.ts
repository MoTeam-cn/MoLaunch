/**
 * 物品搜索工具：ID / 英文名 / 中文名 / 中文全拼统一匹配
 *
 * 依赖 pinyin-pro 做汉字→无声调全拼转换，结果按中文名缓存，
 * 供成就生成器 / 合成配方调色板 / 指令生成器复用同一套匹配逻辑。
 */
import { pinyin } from 'pinyin-pro'

const zhPyCache = new Map<string, string>()

function zhToPinyin(text: string): string {
  if (!text) return ''
  const cached = zhPyCache.get(text)
  if (cached !== undefined) return cached
  const value = pinyin(text, { toneType: 'none', type: 'array' }).join('')
  zhPyCache.set(text, value)
  return value
}

/** 物品的可搜索文本（id + 英文名 + 中文名 + 中文全拼，小写） */
export function itemSearchText(item: { id: string; name: string; zh: string }): string {
  return `${item.id} ${item.name} ${item.zh} ${zhToPinyin(item.zh)}`.toLowerCase()
}

/** 判断物品是否匹配查询词（id / 英文名 / 中文名 / 拼音子串匹配） */
export function matchItem(item: { id: string; name: string; zh: string }, query: string): boolean {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return itemSearchText(item).includes(q)
}
