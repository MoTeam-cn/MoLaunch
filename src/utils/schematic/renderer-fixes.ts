/**
 * 渲染器的两处修正
 *
 * 1. 图集每个方块只占 16×16，开着 mipmap 时远处采样会混进相邻方块的像素，
 *    大片同种方块的地面会浮出一层规则条纹——像素风必须用最近邻采样。
 * 2. 近远裁剪面跟着观察距离走，固定值在几百格距离上深度精度不足。
 *
 * 说明：片元透明度阈值不要再动。库里的 0.01 是它自己的取值，改高会把贴图里
 * 本就偏透明的像素整块丢掉，方块表面会露出背后的天空。
 */
import * as THREE from 'three'
import type { ThreeStructureRenderer } from '@mattzh72/lodestone'

type RendererInternals = {
  atlasTexture?: THREE.Texture
  camera?: THREE.PerspectiveCamera
}

/** 像素风必须最近邻采样，且不能开 mipmap */
export function fixAtlasSampling(renderer: ThreeStructureRenderer): void {
  const texture = (renderer as unknown as RendererInternals).atlasTexture
  if (!texture) return
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter
  texture.generateMipmaps = false
  texture.needsUpdate = true
}

/** 近远裁剪面跟着观察距离走：固定 0.1/2000 在几百格距离上深度精度不足，相邻面会互相啃 */
export function syncClipPlanes(renderer: ThreeStructureRenderer, radius: number): void {
  const camera = (renderer as unknown as RendererInternals).camera
  if (!camera) return
  camera.near = Math.max(0.05, radius / 400)
  camera.far = radius * 12 + 512
  camera.updateProjectionMatrix()
}
