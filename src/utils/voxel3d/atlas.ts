/**
 * 纹理图集与资源包数据组装
 *
 * 把资源包提供的 blockstates / models / textures / 方块标记整理成渲染需要的
 * PackData，并把图集像素喂给 three.js。像素风必须最近邻采样、不生成 mipmap，
 * 否则远处会混进相邻方块的像素。
 */
import * as THREE from 'three'
import type { PackData, TextureRect } from './types'

/** 图集原始像素（来自资源包加载器） */
export interface AtlasSource {
  data: Uint8ClampedArray | Uint8Array
  width: number
  height: number
}

/** 资源包加载器给出的原始数据 */
export interface RawPack {
  blockstates: Record<string, unknown>
  models: Record<string, unknown>
  textures: Record<string, TextureRect>
  atlas: AtlasSource
  opaqueBlocks: Set<string> | readonly string[]
  transparentBlocks: Set<string> | readonly string[]
}

function toSet(value: Set<string> | readonly string[]): ReadonlySet<string> {
  return value instanceof Set ? value : new Set(value)
}

/** 组装成渲染用的 PackData */
export function createPack(raw: RawPack): PackData {
  return {
    blockstates: raw.blockstates,
    models: raw.models,
    textures: raw.textures,
    atlasSize: [raw.atlas.width, raw.atlas.height],
    opaqueBlocks: toSet(raw.opaqueBlocks),
    transparentBlocks: toSet(raw.transparentBlocks),
  }
}

/** 图集像素 → three.js 纹理 */
export function createAtlasTexture(atlas: AtlasSource): THREE.DataTexture {
  // 复制到独立的 ArrayBuffer：DataTexture 要的是 ArrayBuffer 视图，
  // 而源数据可能来自 SharedArrayBuffer 或带偏移的缓冲，直接传类型对不上
  const bytes = new Uint8Array(atlas.data.length)
  bytes.set(atlas.data)
  const texture = new THREE.DataTexture(bytes, atlas.width, atlas.height, THREE.RGBAFormat)
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter
  texture.generateMipmaps = false
  texture.flipY = false
  texture.colorSpace = THREE.SRGBColorSpace
  texture.needsUpdate = true
  return texture
}
