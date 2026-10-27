/**
 * 体素网格生成
 *
 * 遍历体素、按模型展开面、剔除被不透明邻居完全遮住的面，输出两组可直接灌进
 * BufferGeometry 的定点数据：不透明与半透明分开，前者可以写深度、后者需要混合。
 * 纯函数，不碰任何渲染上下文。
 */
import { buildBlockGeometry } from './geometry'
import type { BlockGeometry, MeshData, PackData, VoxelVolume } from './types'
import { DIRECTIONS } from './types'

/** 调用方可复用的缓存，跨多次生成共享能省下大量模型解析 */
export interface MesherCache {
  geometry: Map<string, BlockGeometry>
}

/** 分离后的网格 */
export interface VoxelMesh {
  opaque: MeshData
  translucent: MeshData
  /** 可见面总数 */
  faceCount: number
}

export function createMesherCache(): MesherCache {
  return { geometry: new Map() }
}

const EMPTY_MESH: MeshData = {
  positions: new Float32Array(0),
  normals: new Float32Array(0),
  uvs: new Float32Array(0),
  colors: new Float32Array(0),
  indices: new Uint32Array(0),
  faceCount: 0,
}

/** 某个格子里是不是不透明方块（越界一律当作不遮挡，保证边缘有面） */
function isOpaque(
  volume: VoxelVolume,
  opaqueByIndex: Uint8Array,
  x: number,
  y: number,
  z: number,
): boolean {
  const [sx, sy, sz] = volume.size
  if (x < 0 || y < 0 || z < 0 || x >= sx || y >= sy || z >= sz) return false
  return opaqueByIndex[volume.blocks[x + z * sx + y * sx * sz]] === 1
}

/** 把体素展开成三角网格 */
export function meshVolume(
  volume: VoxelVolume,
  pack: PackData,
  cache: MesherCache = createMesherCache(),
): VoxelMesh {
  const [sx, sy, sz] = volume.size
  const airIndex = volume.airIndex ?? -1

  const opaqueByIndex = new Uint8Array(volume.palette.length)
  for (let i = 0; i < volume.palette.length; i++) {
    const name = volume.palette[i].replace(/^minecraft:/, '').split('[')[0] ?? ''
    opaqueByIndex[i] = pack.opaqueBlocks.has(name) ? 1 : 0
  }

  // 第一遍统计，按需精确分配，避免反复扩容
  const counts = { opaque: 0, translucent: 0 }
  for (let y = 0; y < sy; y++) {
    for (let z = 0; z < sz; z++) {
      for (let x = 0; x < sx; x++) {
        const id = volume.blocks[x + z * sx + y * sx * sz]
        if (id === airIndex) continue
        const state = volume.palette[id]
        if (!state) continue
        const geometry = buildBlockGeometry(state, pack, cache.geometry)
        const bucket = geometry.translucent ? 'translucent' : 'opaque'
        for (const face of geometry.faces) {
          if (face.cullface) {
            const [dx, dy, dz] = DIRECTIONS[face.cullface]
            if (isOpaque(volume, opaqueByIndex, x + dx, y + dy, z + dz)) continue
          }
          counts[bucket]++
        }
      }
    }
  }

  const buffers = {
    opaque: {
      positions: new Float32Array(counts.opaque * 12),
      normals: new Float32Array(counts.opaque * 12),
      uvs: new Float32Array(counts.opaque * 8),
      colors: new Float32Array(counts.opaque * 12),
      indices: new Uint32Array(counts.opaque * 6),
      cursor: 0,
      vertexBase: 0,
    },
    translucent: {
      positions: new Float32Array(counts.translucent * 12),
      normals: new Float32Array(counts.translucent * 12),
      uvs: new Float32Array(counts.translucent * 8),
      colors: new Float32Array(counts.translucent * 12),
      indices: new Uint32Array(counts.translucent * 6),
      cursor: 0,
      vertexBase: 0,
    },
  }

  // 第二遍填充
  for (let y = 0; y < sy; y++) {
    for (let z = 0; z < sz; z++) {
      for (let x = 0; x < sx; x++) {
        const id = volume.blocks[x + z * sx + y * sx * sz]
        if (id === airIndex) continue
        const state = volume.palette[id]
        if (!state) continue
        const geometry = buildBlockGeometry(state, pack, cache.geometry)
        const target = geometry.translucent ? buffers.translucent : buffers.opaque
        for (const face of geometry.faces) {
          if (face.cullface) {
            const [dx, dy, dz] = DIRECTIONS[face.cullface]
            if (isOpaque(volume, opaqueByIndex, x + dx, y + dy, z + dz)) continue
          }
          const pBase = target.cursor * 12
          const uvBase = target.cursor * 8
          for (let v = 0; v < 4; v++) {
            const vertex = face.vertices[v]
            target.positions[pBase + v * 3] = vertex[0] + x
            target.positions[pBase + v * 3 + 1] = vertex[1] + y
            target.positions[pBase + v * 3 + 2] = vertex[2] + z
            target.normals[pBase + v * 3] = face.normal[0]
            target.normals[pBase + v * 3 + 1] = face.normal[1]
            target.normals[pBase + v * 3 + 2] = face.normal[2]
          }
          for (let v = 0; v < 4; v++) {
            const corner = face.uvCorners[v]
            target.uvs[uvBase + v * 2] = corner[0]
            target.uvs[uvBase + v * 2 + 1] = corner[1]
            target.colors[pBase + v * 3] = face.tint ? face.tint[0] : 1
            target.colors[pBase + v * 3 + 1] = face.tint ? face.tint[1] : 1
            target.colors[pBase + v * 3 + 2] = face.tint ? face.tint[2] : 1
          }

          const iBase = target.cursor * 6
          const v0 = target.vertexBase
          target.indices[iBase] = v0
          target.indices[iBase + 1] = v0 + 1
          target.indices[iBase + 2] = v0 + 2
          target.indices[iBase + 3] = v0
          target.indices[iBase + 4] = v0 + 2
          target.indices[iBase + 5] = v0 + 3

          target.vertexBase += 4
          target.cursor++
        }
      }
    }
  }

  const toMesh = (b: typeof buffers.opaque): MeshData =>
    b.cursor === 0
      ? EMPTY_MESH
      : {
          positions: b.positions,
          normals: b.normals,
          uvs: b.uvs,
          colors: b.colors,
          indices: b.indices,
          faceCount: b.cursor,
        }

  return {
    opaque: toMesh(buffers.opaque),
    translucent: toMesh(buffers.translucent),
    faceCount: counts.opaque + counts.translucent,
  }
}
