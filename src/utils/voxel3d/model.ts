/**
 * 方块模型解析
 *
 * 把 blockstate + 模型 json 还原成可以直接生成三角形的一组面（MeshFace）。
 * 处理 parent 继承、贴图变量、面 UV、blockstate 的整模型旋转。
 *
 * 方块坐标约定与 Minecraft 一致：模型空间 0-16，一个方块 = 1.0 世界单位。
 */
import type { FaceDirection } from './types'

interface StateRef {
  name: string
  props: Record<string, string>
}

interface VariantRef {
  model: string
  x?: number
  y?: number
}

interface RawFace {
  texture?: string
  uv?: [number, number, number, number]
  cullface?: FaceDirection
  rotation?: number
  tintindex?: number
}

interface RawElement {
  from: [number, number, number]
  to: [number, number, number]
  faces?: Partial<Record<FaceDirection, RawFace>>
}

interface RawModel {
  parent?: string
  textures?: Record<string, string>
  elements?: RawElement[]
}

/** 「minecraft:oak_stairs[facing=north,half=bottom]」→ 名字与属性 */
export function parseBlockState(state: string): StateRef {
  const bracket = state.indexOf('[')
  if (bracket < 0) return { name: stripNamespace(state.trim()), props: {} }
  const name = stripNamespace(state.slice(0, bracket).trim())
  const props: Record<string, string> = {}
  const body = state.slice(bracket + 1, state.lastIndexOf(']'))
  for (const pair of body.split(',')) {
    const eq = pair.indexOf('=')
    if (eq < 0) continue
    props[pair.slice(0, eq).trim()] = pair.slice(eq + 1).trim()
  }
  return { name, props }
}

export function stripNamespace(id: string): string {
  return id.replace(/^minecraft:/, '')
}

/** 按类型缩小 json 对象，避免到处写断言 */
function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null
}

/** 挑出与属性匹配的变体；不匹配时回退第一个 */
function selectVariant(blockstate: unknown, props: Record<string, string>): VariantRef | null {
  const root = asRecord(blockstate)
  const variants = asRecord(root?.variants)
  if (!variants) return null
  const entries = Object.entries(variants)
  if (entries.length === 0) return null

  for (const [key, raw] of entries) {
    if (key !== '' && !variantMatches(key, props)) continue
    const picked = Array.isArray(raw) ? raw[0] : raw
    const obj = asRecord(picked)
    if (!obj || typeof obj.model !== 'string') continue
    return {
      model: obj.model,
      x: typeof obj.x === 'number' ? obj.x : undefined,
      y: typeof obj.y === 'number' ? obj.y : undefined,
    }
  }
  // 没有精确匹配时用第一条，宁可显示成近似造型也不要整块消失
  const first = Array.isArray(entries[0][1]) ? entries[0][1][0] : entries[0][1]
  const obj = asRecord(first)
  if (!obj || typeof obj.model !== 'string') return null
  return {
    model: obj.model,
    x: typeof obj.x === 'number' ? obj.x : undefined,
    y: typeof obj.y === 'number' ? obj.y : undefined,
  }
}

function variantMatches(key: string, props: Record<string, string>): boolean {
  for (const pair of key.split(',')) {
    const eq = pair.indexOf('=')
    if (eq < 0) continue
    const name = pair.slice(0, eq).trim()
    const value = pair.slice(eq + 1).trim()
    if (props[name] !== value) return false
  }
  return true
}

/** 沿 parent 链合并模型：贴图逐层覆盖，elements 取最靠近子级的那份 */
function resolveModel(id: string, models: Readonly<Record<string, unknown>>, depth = 0): RawModel | null {
  if (depth > 16) return null
  const own = asRecord(models[stripNamespace(id)])
  if (!own) return null
  const parentId = typeof own.parent === 'string' ? own.parent : null
  const parent = parentId ? resolveModel(parentId, models, depth + 1) : null
  const ownTextures = asRecord(own.textures)
  const textures: Record<string, string> = { ...(parent?.textures ?? {}) }
  if (ownTextures) {
    for (const [k, v] of Object.entries(ownTextures)) {
      if (typeof v === 'string') textures[k] = v
    }
  }
  const elements = Array.isArray(own.elements)
    ? (own.elements as RawElement[])
    : (parent?.elements ?? [])
  return { textures, elements }
}

/** 「#all」沿着模型自己的 textures 解析到最终贴图 id */
function resolveTextureRef(ref: string, textures: Record<string, string>, depth = 0): string | null {
  if (!ref.startsWith('#')) return stripNamespace(ref)
  if (depth > 8) return null
  const next = textures[ref.slice(1)]
  if (!next) return null
  return resolveTextureRef(next, textures, depth + 1)
}
export { selectVariant, resolveModel, resolveTextureRef }
export type { RawElement, RawModel, VariantRef }
