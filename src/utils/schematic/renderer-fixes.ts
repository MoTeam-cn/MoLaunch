/**
 * 渲染器的两处修正
 *
 * 1. 图集每个方块只占 16×16，开着 mipmap 时远处采样会混进相邻方块的像素，
 *    大片同种方块的地面会浮出一层规则条纹——像素风必须用最近邻采样。
 * 2. 片元透明度阈值只判 0.01，树叶、草贴图的半透明边缘会被硬画成实心，
 *    树冠和地面上散着一层毛刺。参考实现用的是 0.08。
 */
import * as THREE from 'three'
import type { ThreeStructureRenderer } from '@mattzh72/lodestone'

type RendererInternals = {
  atlasTexture?: THREE.Texture
  opaqueMaterial?: THREE.RawShaderMaterial
  transparentMaterial?: THREE.RawShaderMaterial
  camera?: THREE.PerspectiveCamera
}

export function alignMaterials(renderer: ThreeStructureRenderer): void {
  const internals = renderer as unknown as RendererInternals

  const texture = internals.atlasTexture
  if (texture) {
    texture.magFilter = THREE.NearestFilter
    texture.minFilter = THREE.NearestFilter
    texture.generateMipmaps = false
    texture.needsUpdate = true
  }

  for (const material of [internals.opaqueMaterial, internals.transparentMaterial]) {
    if (!material || typeof material.fragmentShader !== 'string') continue
    if (!material.fragmentShader.includes('texColor.a < 0.01')) continue
    material.fragmentShader = material.fragmentShader.replace(
      'texColor.a < 0.01',
      'texColor.a < 0.08',
    )
    material.needsUpdate = true
  }
}

/** 近远裁剪面跟着观察距离走：固定 0.1/2000 在几百格距离上深度精度不足，相邻面会互相啃 */
export function syncClipPlanes(renderer: ThreeStructureRenderer, radius: number): void {
  const camera = (renderer as unknown as RendererInternals).camera
  if (!camera) return
  camera.near = Math.max(0.05, radius / 400)
  camera.far = radius * 12 + 512
  camera.updateProjectionMatrix()
}
