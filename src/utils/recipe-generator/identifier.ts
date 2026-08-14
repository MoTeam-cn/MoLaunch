/**
 * 合成配方生成器 - 标识符解析
 */

export type ItemRef = {
  namespace: string
  id: string
  data?: number
}

const NAMESPACE_REGEX = /^[a-z0-9._-]+$/
const ID_REGEX = /^[a-z0-9._/-]+$/

export function parseIdentifier(raw: string): ItemRef | null {
  if (!raw) return null
  const value = raw.trim()
  if (!value) return null
  let namespace = 'minecraft'
  let id = value
  let data: number | undefined

  // data 后缀（1.12 旧版，如 minecraft:stone:1）
  const dataMatch = value.match(/^(.*):(\d+)$/)
  if (dataMatch) {
    id = dataMatch[1]
    data = Number(dataMatch[2])
  }

  const nsIndex = id.indexOf(':')
  if (nsIndex !== -1) {
    namespace = id.slice(0, nsIndex)
    id = id.slice(nsIndex + 1)
  }
  if (!NAMESPACE_REGEX.test(namespace) || !ID_REGEX.test(id)) return null
  return { namespace, id, data }
}

export function parseItemId(raw: string): ItemRef | null {
  return parseIdentifier(raw)
}

export function itemRefToString(ref: ItemRef): string {
  const base = `${ref.namespace}:${ref.id}`
  return ref.data !== undefined ? `${base}:${ref.data}` : base
}

export function stripData(raw: string): string {
  return raw.replace(/:\d+$/, '')
}

export function isValidIdentifier(raw: string): boolean {
  return parseIdentifier(raw) !== null
}
