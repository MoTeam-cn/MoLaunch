/**
 * 合成配方生成器 - 标识符解析（对应规格 §3.2：物品引用与 data 后缀）
 *
 * 支持三种写法：`stone`、`minecraft:stone`、`minecraft:stone:1`（1.12 的 data 值）。
 */

export type ItemRef = {
  namespace: string
  id: string
  /** 1.12 的数据值后缀 */
  data?: number
}

/** 命名空间与路径的合法字符（Minecraft 资源位置规则） */
const NAMESPACE_PATTERN = /^[a-z0-9._-]+$/
const PATH_PATTERN = /^[a-z0-9._/-]+$/
const DATA_SUFFIX = /^(.*):(\d+)$/

export function parseIdentifier(raw: string): ItemRef | null {
  const value = raw.trim()
  if (!value) return null

  // 先剥掉末尾的数字后缀，再切命名空间
  const withData = DATA_SUFFIX.exec(value)
  const body = withData ? withData[1] : value
  const data = withData ? Number(withData[2]) : undefined

  const separator = body.indexOf(':')
  const namespace = separator === -1 ? 'minecraft' : body.slice(0, separator)
  const id = separator === -1 ? body : body.slice(separator + 1)

  if (!NAMESPACE_PATTERN.test(namespace) || !PATH_PATTERN.test(id)) return null
  return data === undefined ? { namespace, id } : { namespace, id, data }
}

export function parseItemId(raw: string): ItemRef | null {
  return parseIdentifier(raw)
}

export function itemRefToString(ref: ItemRef): string {
  const head = `${ref.namespace}:${ref.id}`
  return ref.data === undefined ? head : `${head}:${ref.data}`
}

export function stripData(raw: string): string {
  return raw.replace(/:\d+$/, '')
}

export function isValidIdentifier(raw: string): boolean {
  return parseIdentifier(raw) !== null
}
