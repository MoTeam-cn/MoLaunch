/**
 * 体素渲染的公共类型
 *
 * 数据一律用「方块状态字符串 + 扁平索引」表达，不绑定任何具体的渲染库。
 */

/** 图集里一块贴图的像素矩形 [x, y, w, h] */
export type TextureRect = [number, number, number, number]

/** 体素数据：index = x + z * sx + y * sx * sz */
export interface VoxelVolume {
  size: [number, number, number]
  blocks: ArrayLike<number>
  /** 方块状态字符串，下标即 blocks 里的值 */
  palette: readonly string[]
  /** 该调色板下标视为空气，生成网格时跳过 */
  airIndex?: number
}

/** 资源包提供的数据（只读，不含渲染逻辑） */
export interface PackData {
  /** blockstate 定义，键为方块名（不带命名空间） */
  blockstates: Readonly<Record<string, unknown>>
  /** 模型定义，键为 block/model_name 形式 */
  models: Readonly<Record<string, unknown>>
  /** 贴图 → 图集矩形 */
  textures: Readonly<Record<string, TextureRect>>
  /** 图集尺寸（像素） */
  atlasSize: [number, number]
  /** 完全遮挡视线的方块（可剔除相邻面） */
  opaqueBlocks: ReadonlySet<string>
  /** 需要半透明混合的方块 */
  transparentBlocks: ReadonlySet<string>
}

/** 模型里一个面的朝向 */
export type FaceDirection = 'north' | 'south' | 'west' | 'east' | 'up' | 'down'

/** 解析后的单个面（已换算到 0-1 的方块局部坐标与 UV） */
export interface MeshFace {
  /** 四个顶点，逆时针朝外 */
  vertices: [number, number, number][]
  /** 四个顶点各自的 UV，与 vertices 一一对应，已归一化到 0-1 */
  uvCorners: [number, number][]
  /** 贴图 UV 的包围盒 [u0, v0, u1, v1]，便于校验与调试 */
  uv: [number, number, number, number]
  /** 该面朝哪一侧；有值时邻居不透明即可跳过 */
  cullface?: FaceDirection
  /** 生物群系着色，null 表示按贴图原色 */
  tint?: [number, number, number] | null
  /** 法线 */
  normal: [number, number, number]
}

/** 一个方块模型拆出来的全部面 */
export interface BlockGeometry {
  faces: MeshFace[]
  /** 该方块是否含需要半透明混合的贴图 */
  translucent: boolean
}

/** 生成好的网格数据，可直接灌进 BufferGeometry */
export interface MeshData {
  positions: Float32Array
  normals: Float32Array
  uvs: Float32Array
  /** 每个顶点的着色（0-1），未着色的方块为白色 */
  colors: Float32Array
  /** 每个面的中心点（每面 3 个分量），半透明网格按它排序 */
  centers: Float32Array
  indices: Uint32Array
  /** 面数 */
  faceCount: number
}

/** 六个方向与单位偏移 */
export const DIRECTIONS: Readonly<Record<FaceDirection, [number, number, number]>> = {
  east: [1, 0, 0],
  west: [-1, 0, 0],
  up: [0, 1, 0],
  down: [0, -1, 0],
  south: [0, 0, 1],
  north: [0, 0, -1],
}
