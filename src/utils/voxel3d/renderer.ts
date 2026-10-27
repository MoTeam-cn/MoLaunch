/**
 * 体素渲染器
 *
 * 一个 canvas 显示一份体素数据：生成网格、建场景、跑渲染循环。相机与交互由外部
 * 传入（见 orbit.ts），这里只负责把数据画出来。
 *
 * 不透明与半透明分成两个网格：前者写深度，后者混合且不写深度，避免互相遮挡出错。
 */
import * as THREE from 'three'
import { createAtlasTexture, type AtlasSource } from './atlas'
import { meshVolume, type MesherCache, createMesherCache } from './mesher'
import type { PackData, VoxelVolume } from './types'
import { FOV } from './orbit'

export interface VoxelRendererOptions {
  antialias?: boolean
  /** 天空/背景色 */
  background?: string
  /** 环境光强度 */
  ambientIntensity?: number
}

/** 贴图里 alpha 低于这个值的像素直接丢弃：树叶、草的边缘要靠它 */
const ALPHA_TEST = 0.5

export class VoxelRenderer {
  private readonly renderer: THREE.WebGLRenderer
  private readonly scene = new THREE.Scene()
  private readonly camera: THREE.PerspectiveCamera
  private readonly pack: PackData
  private readonly cache: MesherCache = createMesherCache()
  private readonly opaqueMaterial: THREE.MeshLambertMaterial
  private readonly translucentMaterial: THREE.MeshLambertMaterial
  private opaqueMesh: THREE.Mesh | null = null
  private translucentMesh: THREE.Mesh | null = null
  private rafId = 0
  private disposed = false
  private faceCount = 0

  constructor(canvas: HTMLCanvasElement, pack: PackData, atlas: AtlasSource, options: VoxelRendererOptions = {}) {
    this.pack = pack
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: options.antialias ?? true,
      alpha: false,
    })
    this.renderer.setClearColor(new THREE.Color(options.background ?? '#12161f'), 1)

    this.camera = new THREE.PerspectiveCamera(FOV, 1, 0.1, 2000)

    const texture = createAtlasTexture(atlas)
    this.opaqueMaterial = new THREE.MeshLambertMaterial({
      map: texture,
      alphaTest: ALPHA_TEST,
      transparent: false,
      vertexColors: true,
      side: THREE.FrontSide,
    })
    this.translucentMaterial = new THREE.MeshLambertMaterial({
      map: texture,
      alphaTest: 0.05,
      transparent: true,
      opacity: 0.75,
      depthWrite: false,
      vertexColors: true,
      side: THREE.DoubleSide,
    })

    // 环境光保证背光面不死黑，方向光给出方块之间的明暗差异
    this.scene.add(new THREE.AmbientLight(0xffffff, options.ambientIntensity ?? 2.6))
    const sun = new THREE.DirectionalLight(0xffffff, 1.5)
    sun.position.set(0.6, 1, 0.35)
    this.scene.add(sun)
  }

  /** 换一份体素数据（例如切换渲染模式） */
  setVolume(volume: VoxelVolume): number {
    const mesh = meshVolume(volume, this.pack, this.cache)
    this.disposeMeshes()

    if (mesh.opaque.faceCount > 0) {
      this.opaqueMesh = new THREE.Mesh(makeGeometry(mesh.opaque), this.opaqueMaterial)
      this.scene.add(this.opaqueMesh)
    }
    if (mesh.translucent.faceCount > 0) {
      this.translucentMesh = new THREE.Mesh(makeGeometry(mesh.translucent), this.translucentMaterial)
      this.translucentMesh.renderOrder = 1
      this.scene.add(this.translucentMesh)
    }

    this.faceCount = mesh.faceCount
    return this.faceCount
  }

  getFaceCount(): number {
    return this.faceCount
  }

  setViewport(width: number, height: number, pixelRatio = 1): void {
    const w = Math.max(width, 1)
    const h = Math.max(height, 1)
    this.renderer.setPixelRatio(pixelRatio)
    this.renderer.setSize(w, h, false)
    this.camera.aspect = w / h
    this.camera.updateProjectionMatrix()
  }

  setCamera(position: [number, number, number], target: [number, number, number]): void {
    this.camera.position.set(position[0], position[1], position[2])
    this.camera.lookAt(target[0], target[1], target[2])
    // 近远面跟着观察距离走，固定值在几百格上深度精度不够，相邻面会互相啃
    const distance = this.camera.position.distanceTo(
      new THREE.Vector3(target[0], target[1], target[2]),
    )
    this.camera.near = Math.max(0.05, distance / 400)
    this.camera.far = distance * 12 + 512
    this.camera.updateProjectionMatrix()
  }

  setFov(fov: number): void {
    this.camera.fov = fov
    this.camera.updateProjectionMatrix()
  }

  renderOnce(): void {
    if (this.disposed) return
    this.renderer.render(this.scene, this.camera)
  }

  start(): void {
    if (this.rafId) return
    const loop = () => {
      if (this.disposed) return
      this.renderOnce()
      this.rafId = requestAnimationFrame(loop)
    }
    this.rafId = requestAnimationFrame(loop)
  }

  stop(): void {
    if (this.rafId) cancelAnimationFrame(this.rafId)
    this.rafId = 0
  }

  private disposeMeshes(): void {
    for (const mesh of [this.opaqueMesh, this.translucentMesh]) {
      if (!mesh) continue
      this.scene.remove(mesh)
      mesh.geometry.dispose()
    }
    this.opaqueMesh = null
    this.translucentMesh = null
  }

  dispose(): void {
    this.disposed = true
    this.stop()
    this.disposeMeshes()
    this.opaqueMaterial.map?.dispose()
    this.opaqueMaterial.dispose()
    this.translucentMaterial.dispose()
    this.renderer.dispose()
  }
}

function makeGeometry(mesh: {
  positions: Float32Array
  normals: Float32Array
  uvs: Float32Array
  colors: Float32Array
  indices: Uint32Array
}): THREE.BufferGeometry {
  const geometry = new THREE.BufferGeometry()
  geometry.setAttribute('position', new THREE.BufferAttribute(mesh.positions, 3))
  geometry.setAttribute('normal', new THREE.BufferAttribute(mesh.normals, 3))
  geometry.setAttribute('uv', new THREE.BufferAttribute(mesh.uvs, 2))
  geometry.setAttribute('color', new THREE.BufferAttribute(mesh.colors, 3))
  geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1))
  geometry.computeBoundingSphere()
  return geometry
}
