/**
 * 资源包 3D 预览 - lodestone Resources 构建
 *
 * 将 rp_read_many 返回的模型/纹理文件与原版内置资源合并：
 * - 模型/blockstate：资源包优先，缺失时回退原版（flatten 时 parent 自动补 minecraft 命名空间）
 * - 纹理：pack 覆盖同 id 区域 / 新增 id 分配扩展区，烘焙进原版纹理图集；未知 id 回退占位格
 * - 入口：blockstate 取第一个模型引用，模型文件直接作为渲染目标（合成 `""` 变体）
 *
 * 原版内置资源（atlas/assets/flags）通过 Vite `?url` 从 lodestone 包内静态导入，
 * 生产构建会为每个文件生成独立哈希资源，故此处不复用 loadDefaultPackResources 的 baseUrl 推导。
 */
import {
  BlockDefinition,
  BlockModel,
  createResourcesFromPack,
  upperPowerOfTwo,
} from '@mattzh72/lodestone'
import type {
  BlockFlagsProvider,
  BlockModelProvider,
  BlockPropertiesProvider,
  Resources,
  TextureAtlasProvider,
} from '@mattzh72/lodestone'
import atlasPngUrl from '@mattzh72/lodestone/assets/default-pack/atlas.png?url'
import assetsJsonUrl from '@mattzh72/lodestone/assets/default-pack/assets.json?url'
import nonSelfCullingTxtUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/non_self_culling.txt?url'
import opaqueTxtUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/opaque.txt?url'
import transparentTxtUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/transparent.txt?url'
import emissiveJsonUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/emissive.json?url'

/** rp_read_many 返回的单文件条目 */
export interface RpPreviewFile {
  kind: string
  content: string
}

/** 渲染目标的合成方块 id（模型文件预览时无对应 blockstate） */
const PREVIEW_BLOCK = 'molaunch:preview'

interface VanillaPackBase {
  assets: {
    blockstates: Record<string, unknown>
    models: Record<string, unknown>
    textures: Record<string, [number, number, number, number]>
  }
  atlas: { imageData: ImageData; atlasSize: number }
  flags: {
    opaque: Set<string>
    transparent: Set<string>
    nonSelfCulling: Set<string>
    emissive: Record<string, { intensity?: number; conditional?: string }>
  }
  resources: Resources
}

let vanillaBasePromise: Promise<VanillaPackBase> | null = null

/** 原版内置资源（模块级缓存，复用同一次会话） */
export function getVanillaBase(): Promise<VanillaPackBase> {
  if (!vanillaBasePromise) vanillaBasePromise = loadVanillaBase()
  return vanillaBasePromise
}

async function loadVanillaBase(): Promise<VanillaPackBase> {
  const [assetsRes, atlasRes, opaqueRes, transparentRes, nonSelfCullingRes, emissiveRes] =
    await Promise.all([
      fetch(assetsJsonUrl),
      fetch(atlasPngUrl),
      fetch(opaqueTxtUrl),
      fetch(transparentTxtUrl),
      fetch(nonSelfCullingTxtUrl),
      fetch(emissiveJsonUrl),
    ])
  const resList = [
    assetsRes,
    atlasRes,
    opaqueRes,
    transparentRes,
    nonSelfCullingRes,
    emissiveRes,
  ]
  const failed = resList.find((r) => !r.ok)
  if (failed) throw new Error(`原版内置资源加载失败: ${failed.status} ${failed.statusText}`)
  const assets = await assetsRes.json()
  // lodestone 只特判 builtin/generated；内置 item 模板（如 item/milk_bucket）的 parent 是 builtin/entity，
  // 缺失会导致其 flatten 时被静默清空、引用它们的模型渲染黑屏。注入等价定义使其生成 layer0 平面。
  assets.models ??= {}
  assets.models['builtin/entity'] = { parent: 'builtin/generated' }
  const atlas = await decodeAtlasToImageData(await atlasRes.blob())
  const flags = {
    opaque: parseBlockList(await opaqueRes.text()),
    transparent: parseBlockList(await transparentRes.text()),
    nonSelfCulling: parseBlockList(await nonSelfCullingRes.text()),
    emissive: (await emissiveRes.json()) as Record<string, { intensity?: number; conditional?: string }>,
  }
  return { assets, atlas, flags, resources: createResourcesFromPack({ assets, atlas, flags }) }
}

async function decodeAtlasToImageData(
  atlasBlob: Blob,
): Promise<{ imageData: ImageData; atlasSize: number }> {
  const bitmap = typeof createImageBitmap === 'function' ? await createImageBitmap(atlasBlob) : null
  const w = bitmap?.width ?? 0
  const h = bitmap?.height ?? 0
  if (!bitmap || w <= 0 || h <= 0) throw new Error('无法解码原版纹理图集 atlas.png')
  const atlasSize = upperPowerOfTwo(Math.max(w, h))
  const canvas =
    typeof OffscreenCanvas !== 'undefined'
      ? new OffscreenCanvas(atlasSize, atlasSize)
      : Object.assign(document.createElement('canvas'), { width: atlasSize, height: atlasSize })
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('无法创建 2D 画布解码纹理图集')
  ctx.drawImage(bitmap, 0, 0)
  return { imageData: ctx.getImageData(0, 0, atlasSize, atlasSize), atlasSize }
}

function parseBlockList(text: string): Set<string> {
  const ids = new Set<string>()
  const matches = text.match(/minecraft:[a-z0-9_]+/g) ?? []
  matches.forEach((match) => ids.add(match))
  text
    .split(/\s+/)
    .map((token) => token.trim())
    .filter(Boolean)
    .forEach((token) => {
      ids.add(token.startsWith('minecraft:') ? token : `minecraft:${token}`)
    })
  return ids
}

/** 从 rel_path 解析 id：assets/<ns>/<cat>/<path>.(json|png) -> <ns>:<path> */
function relToId(rel: string): string | null {
  const m = /^assets\/([^/]+)\/(?:blockstates|models|textures)\/(.+)\.(?:json|png)$/.exec(rel)
  return m ? `${m[1]}:${m[2]}` : null
}

/** blockstate JSON 中取第一个模型引用（variants 优先，其次 multipart.apply） */
function firstModelRef(def: unknown): string | null {
  if (!def || typeof def !== 'object') return null
  const obj = def as Record<string, unknown>
  const variants = obj.variants
  if (variants && typeof variants === 'object') {
    for (const entry of Object.values(variants as Record<string, unknown>)) {
      const model = pickEntryModel(entry)
      if (model) return model
    }
  }
  const multipart = obj.multipart
  if (Array.isArray(multipart)) {
    for (const part of multipart) {
      const model = pickEntryModel((part as Record<string, unknown>)?.apply)
      if (model) return model
    }
  }
  return null
}

function pickEntryModel(entry: unknown): string | null {
  if (typeof entry === 'string') return entry
  if (Array.isArray(entry)) {
    for (const item of entry) {
      const m = pickEntryModel(item)
      if (m) return m
    }
    return null
  }
  if (entry && typeof entry === 'object') {
    const model = (entry as Record<string, unknown>).model
    return typeof model === 'string' ? model : null
  }
  return null
}

/** 模型引用解析：有命名空间直接用；无命名空间先按文件命名空间、再补 minecraft */
function resolveModelRef(
  ref: string,
  defaultNs: string,
  packModels: Map<string, unknown>,
  vanillaModels: Record<string, unknown>,
): string | null {
  const candidates = ref.includes(':')
    ? [ref]
    : [`${defaultNs}:${ref}`, `minecraft:${ref}`]
  return candidates.find((c) => packModels.has(c) || vanillaModels[c] != null) ?? null
}

/** 沿 parent 链（pack 优先、vanilla 回退）检查模型是否可解析；返回第一个断链的模型 id（null = 链完整） */
function missingParentInChain(
  modelId: string,
  packModels: Map<string, unknown>,
  vanillaModels: Record<string, unknown>,
): string | null {
  const seen = new Set<string>()
  const stack = [modelId]
  while (stack.length) {
    const id = stack.pop()!
    if (seen.has(id)) continue
    seen.add(id)
    // assets key 不带 minecraft: 前缀，查询时回退无前缀形式
    const json = packModels.get(id) ?? vanillaModels[id] ?? vanillaModels[id.slice(id.indexOf(':') + 1)]
    if (json == null || typeof json !== 'object') return id
    const parent = (json as Record<string, unknown>).parent
    if (typeof parent !== 'string') continue
    if (
      parent === 'builtin/generated' ||
      parent === 'minecraft:builtin/generated' ||
      parent === 'builtin/entity' ||
      parent === 'minecraft:builtin/entity'
    )
      continue // lodestone 内置生成平面，无需 parent 文件
    stack.push(parent.includes(':') ? parent : `minecraft:${parent}`)
  }
  return null
}

let fallbackItemModel: BlockModel | null = null

function getFallbackItemModel(): BlockModel {
  if (!fallbackItemModel) fallbackItemModel = BlockModel.fromJson({ parent: 'builtin/generated' })
  return fallbackItemModel
}

/** 模型读取：pack 优先 → vanilla 回退 → 任意缺失 id 以 generated 平面兜底（layer0 纹理仍可显示） */
function buildModelProvider(
  blockModels: Map<string, BlockModel>,
  vanilla: VanillaPackBase,
): BlockModelProvider {
  const warned = new Set<string>()
  return {
    getBlockModel(id) {
      const key = String(id)
      const found = blockModels.get(key) ?? vanilla.resources.getBlockModel(id)
      if (found) return found
      if (!warned.has(key)) {
        warned.add(key)
        console.warn(`[preview] 模型 ${key} 缺失，已回退 generated 平面（带 layer0 纹理仍可显示）`)
      }
      return getFallbackItemModel()
    },
  }
}

/** 构建预览 Resources；返回渲染目标 blockId */
export async function buildPreviewResources(
  files: Map<string, RpPreviewFile>,
  root: string,
): Promise<{ resources: Resources; blockId: string }> {
  const vanilla = await getVanillaBase()

  const packBlockstates = new Map<string, unknown>()
  const packModels = new Map<string, unknown>()
  const packTextures = new Map<string, string>()
  for (const [rel, f] of files) {
    const id = relToId(rel)
    if (!id) continue
    if (rel.includes('/blockstates/')) {
      try {
        packBlockstates.set(id, JSON.parse(f.content) as unknown)
      } catch {
        // 损坏 blockstate 跳过
      }
    } else if (rel.includes('/models/')) {
      try {
        packModels.set(id, JSON.parse(f.content) as unknown)
      } catch {
        // 损坏模型跳过
      }
    } else if (rel.includes('/textures/') && f.kind === 'data_uri') {
      packTextures.set(id, f.content)
    }
  }

  const isBlockstate = root.includes('/blockstates/')
  const rootId = relToId(root)
  let blockId = rootId ?? PREVIEW_BLOCK
  let modelId: string | null

  if (isBlockstate && rootId) {
    const def = packBlockstates.get(rootId)
    const ref = firstModelRef(def)
    if (!ref) throw new Error('blockstate 中没有可用的模型引用')
    const defaultNs = rootId.slice(0, rootId.indexOf(':'))
    modelId = resolveModelRef(ref, defaultNs, packModels, vanilla.assets.models)
    if (!modelId) throw new Error(`blockstate 引用的模型不在资源包内：${ref}`)
  } else {
    modelId = rootId && packModels.has(rootId) ? rootId : null
    if (!modelId) throw new Error('模型文件解析失败或不存在')
    blockId = PREVIEW_BLOCK
  }

  const missing = missingParentInChain(modelId, packModels, vanilla.assets.models)
  if (missing) {
    console.warn(`[preview] parent 链存在缺失模型 ${missing}，已回退 generated 平面渲染`)
  }

  const blockDefinitions = new Map<string, BlockDefinition>()
  blockDefinitions.set(blockId, BlockDefinition.fromJson({ variants: { '': { model: modelId } } }))

  const blockModels = new Map<string, BlockModel>()
  for (const [id, json] of packModels) {
    try {
      blockModels.set(id, BlockModel.fromJson(json))
    } catch {
      // 损坏模型跳过
    }
  }
  const modelAccessor: BlockModelProvider = buildModelProvider(blockModels, vanilla)
  for (const m of blockModels.values()) m.flatten(modelAccessor)

  const target = blockModels.get(modelId) as
    | { elements?: unknown[]; textures?: Record<string, string> }
    | undefined
  const firstEl = target?.elements?.[0] as
    | { from?: number[]; to?: number[]; faces?: Record<string, unknown> }
    | undefined
  console.log(
    `[preview] 渲染模型 ${modelId}：elements=${target?.elements?.length ?? 0}，` +
      `layer0=${target?.textures?.layer0 ?? '(无)'}，packModels=${packModels.size}，packTextures=${packTextures.size}`,
  )
  console.log(
    `[preview] 首元素 from=${JSON.stringify(firstEl?.from)} to=${JSON.stringify(firstEl?.to)} ` +
      `faces=${JSON.stringify(firstEl?.faces ? Object.keys(firstEl.faces) : [])}`,
  )

  const atlas = await mergeAtlas(vanilla, packTextures)
  const layer0 = target?.textures?.layer0
  if (layer0) {
    console.log(`[preview] 纹理 ${layer0} UV=${JSON.stringify(atlas.idMap[layer0] ?? atlas.missingUV)}`)
  }

  return {
    resources: createPreviewResources(atlas, blockDefinitions, blockModels, vanilla),
    blockId,
  }
}

type UV4 = [number, number, number, number]

interface MergedAtlas {
  imageData: ImageData
  /** id -> UV 归一化坐标；unknown id 回退 missingUV */
  idMap: Record<string, UV4>
  missingUV: UV4
}

async function mergeAtlas(vanilla: VanillaPackBase, packTextures: Map<string, string>): Promise<MergedAtlas> {
  const size0 = vanilla.atlas.atlasSize
  let canvas = document.createElement('canvas')
  canvas.width = size0
  canvas.height = size0
  const initialCtx = canvas.getContext('2d')
  if (!initialCtx) throw new Error('无法创建纹理合并画布')
  let ctx: CanvasRenderingContext2D = initialCtx
  ctx.putImageData(vanilla.atlas.imageData, 0, 0)

  let allocY = size0
  let size = size0
  const placements: { id: string; x: number; y: number; w: number; h: number }[] = []

  const place = async (id: string, dataUri: string, w: number, h: number, frameH?: number) => {
    const bitmap = await loadBitmap(dataUri)
    const drawH = frameH ?? h
    if (allocY + drawH + 1 > size) {
      const newSize = upperPowerOfTwo(Math.max(size, allocY + drawH + 1))
      const bigger = document.createElement('canvas')
      bigger.width = newSize
      bigger.height = newSize
      const bctx = bigger.getContext('2d')
      if (!bctx) throw new Error('无法扩展纹理合并画布')
      bctx.drawImage(canvas, 0, 0)
      canvas = bigger
      ctx = bctx
      size = newSize
    }
    ctx.drawImage(bitmap, 0, 0, w, drawH, 0, allocY, w, drawH)
    placements.push({ id, x: 0, y: allocY, w, h: drawH })
    allocY += drawH + 1
  }

  // 16px 占位格（未知纹理回退）+ 全部 pack 纹理
  await place('__missing__', missingTextureDataUri(), 16, 16)
  for (const [id, uri] of packTextures) {
    const [w, h] = await imageSize(uri)
    // Minecraft 垂直动画条（每帧 w×w）：lodestone 不支持动画帧，仅取第一帧，避免整条贴面变形
    const frameH = h > w && h % w === 0 ? w : h
    if (frameH !== h) console.log(`[preview] pack 纹理 ${id} 尺寸 ${w}x${h} 为动画条，仅取第一帧 ${w}x${frameH}`)
    await place(id, uri, w, h, frameH)
  }
  console.log(`[preview] 图集最终尺寸 ${size}`)

  const idMap: Record<string, UV4> = {}
  const tex = vanilla.assets.textures
  for (const id of Object.keys(tex)) {
    const [u, v, du, dv] = tex[id]
    const dv2 = du !== dv && id.startsWith('block/') ? du : dv
    idMap[`minecraft:${id}`] = [u / size, v / size, (u + du) / size, (v + dv2) / size]
  }
  const missingUV = placements.find((p) => p.id === '__missing__')!
  for (const p of placements) {
    if (p.id === '__missing__') continue
    idMap[p.id] = [p.x / size, p.y / size, (p.x + p.w) / size, (p.y + p.h) / size]
  }

  return {
    imageData: ctx.getImageData(0, 0, size, size),
    idMap,
    missingUV: [missingUV.x / size, missingUV.y / size, (missingUV.x + missingUV.w) / size, (missingUV.y + missingUV.h) / size],
  }
}

async function loadBitmap(dataUri: string): Promise<ImageBitmap | HTMLImageElement> {
  const blob = await (await fetch(dataUri)).blob()
  if (typeof createImageBitmap === 'function') return createImageBitmap(blob)
  const url = URL.createObjectURL(blob)
  const img = new Image()
  img.src = url
  await img.decode()
  return img
}

function imageSize(dataUri: string): Promise<[number, number]> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve([img.naturalWidth || 16, img.naturalHeight || 16])
    img.onerror = () => reject(new Error('纹理图片解析失败'))
    img.src = dataUri
  })
}

/** 16×16 品红占位纹理（对应 lodestone 的 missing texture） */
function missingTextureDataUri(): string {
  const canvas = document.createElement('canvas')
  canvas.width = 16
  canvas.height = 16
  const ctx = canvas.getContext('2d')
  if (!ctx) return ''
  ctx.fillStyle = '#ff00ff'
  ctx.fillRect(0, 0, 16, 16)
  ctx.fillStyle = '#000000'
  for (let y = 0; y < 16; y += 8) {
    for (let x = 0; x < 16; x += 8) {
      ctx.fillRect(x, y, 4, 4)
      ctx.fillRect(x + 4, y + 4, 4, 4)
    }
  }
  return canvas.toDataURL('image/png')
}

function createPreviewResources(
  atlas: MergedAtlas,
  blockDefinitions: Map<string, BlockDefinition>,
  blockModels: Map<string, BlockModel>,
  vanilla: VanillaPackBase,
): Resources {
  const modelProvider: BlockModelProvider = buildModelProvider(blockModels, vanilla)
  const flagProvider: BlockFlagsProvider = vanilla.resources
  const textureProvider: TextureAtlasProvider = {
    getTextureAtlas() {
      return atlas.imageData
    },
    getTextureUV(id) {
      return atlas.idMap[String(id)] ?? atlas.missingUV
    },
    getPixelSize() {
      return 16
    },
  }
  const blockPropertiesProvider: BlockPropertiesProvider = vanilla.resources

  return {
    getBlockDefinition(id) {
      const key = String(id)
      return blockDefinitions.get(key) ?? vanilla.resources.getBlockDefinition(id)
    },
    getBlockModel(id) {
      return modelProvider.getBlockModel(id)
    },
    getTextureAtlas() {
      return textureProvider.getTextureAtlas()
    },
    getTextureUV(id) {
      return textureProvider.getTextureUV(id)
    },
    getPixelSize() {
      return 16
    },
    getBlockFlags(id) {
      return flagProvider.getBlockFlags(id)
    },
    getBlockProperties(id) {
      return blockPropertiesProvider.getBlockProperties(id)
    },
    getDefaultBlockProperties(id) {
      return blockPropertiesProvider.getDefaultBlockProperties(id)
    },
  }
}
