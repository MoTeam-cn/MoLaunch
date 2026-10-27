/**
 * 资源包加载
 *
 * 一次性取回方块定义、模型、贴图坐标与图集像素，整理成渲染需要的 PackData。
 * 只读数据，不构造任何模型解析器：本模块的解析在上面 model.ts / geometry.ts 里。
 */
import assetsJsonUrl from '@mattzh72/lodestone/assets/default-pack/assets.json?url'
import atlasPngUrl from '@mattzh72/lodestone/assets/default-pack/atlas.png?url'
import opaqueTxtUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/opaque.txt?url'
import transparentTxtUrl from '@mattzh72/lodestone/assets/default-pack/block-flags/transparent.txt?url'
import { createPack } from './atlas'
import type { AtlasSource } from './atlas'
import type { PackData, TextureRect } from './types'

/** 加载好的资源，pack 与 atlas 配套使用 */
export interface LoadedVoxelPack {
  pack: PackData
  atlas: AtlasSource
}

let cached: Promise<LoadedVoxelPack> | null = null

/** 加载内置资源包（模块级缓存，同一会话只加载一次） */
export function loadVoxelPack(): Promise<LoadedVoxelPack> {
  cached ??= load()
  return cached
}

async function load(): Promise<LoadedVoxelPack> {
  const [assets, atlasBlob, opaque, transparent] = await Promise.all([
    fetchJson<{
      blockstates: Record<string, unknown>
      models: Record<string, unknown>
      textures: Record<string, TextureRect>
    }>(assetsJsonUrl),
    fetchBlob(atlasPngUrl),
    fetchText(opaqueTxtUrl),
    fetchText(transparentTxtUrl),
  ])
  const atlas = await decodeAtlas(atlasBlob)
  const pack = createPack({
    blockstates: assets.blockstates,
    models: assets.models,
    textures: assets.textures,
    atlas,
    opaqueBlocks: parseBlockList(opaque),
    transparentBlocks: parseBlockList(transparent),
  })
  return { pack, atlas }
}

async function fetchJson<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`资源加载失败：${res.status} ${res.statusText}`)
  return (await res.json()) as T
}

async function fetchText(url: string): Promise<string> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`资源加载失败：${res.status} ${res.statusText}`)
  return res.text()
}

async function fetchBlob(url: string): Promise<Blob> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`资源加载失败：${res.status} ${res.statusText}`)
  return res.blob()
}

/** PNG → RGBA 像素 */
async function decodeAtlas(blob: Blob): Promise<AtlasSource> {
  const bitmap = await createImageBitmap(blob)
  const width = bitmap.width
  const height = bitmap.height
  if (width <= 0 || height <= 0) throw new Error('无法解码纹理图集')
  const canvas =
    typeof OffscreenCanvas !== 'undefined'
      ? new OffscreenCanvas(width, height)
      : Object.assign(document.createElement('canvas'), { width, height })
  const ctx = canvas.getContext('2d') as CanvasRenderingContext2D | null
  if (!ctx) throw new Error('无法创建 2D 画布解码纹理图集')
  ctx.drawImage(bitmap, 0, 0)
  bitmap.close()
  const imageData = ctx.getImageData(0, 0, width, height)
  return { data: imageData.data, width, height }
}

/** 一行一个方块名 */
function parseBlockList(text: string): Set<string> {
  const names = new Set<string>()
  for (const line of text.split('\n')) {
    const name = line.trim()
    if (name) names.add(name)
  }
  return names
}
