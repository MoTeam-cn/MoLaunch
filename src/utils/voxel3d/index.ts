/**
 * 体素 3D 渲染
 *
 * 给「方块状态 + 扁平索引」的体素数据渲染可旋转缩放的立体预览。
 * 纯前端、只依赖 three.js：模型解析、面剔除、网格生成、相机全部自己实现，
 * 不依赖特定渲染库的行为，便于其它功能直接复用。
 *
 * 用法：
 *   const pack = createPack(rawPack)
 *   const renderer = new VoxelRenderer(canvas, pack, atlasSource)
 *   renderer.setVolume({ size, blocks, palette, airIndex })
 *   renderer.setCamera(orbitPosition(target, radius, yaw, pitch), target)
 *   renderer.start()
 */
export { VoxelRenderer } from './renderer'
export type { VoxelRendererOptions } from './renderer'
export { loadVoxelPack } from './loader'
export type { LoadedVoxelPack } from './loader'
export { createPack, createAtlasTexture } from './atlas'
export type { AtlasSource, RawPack } from './atlas'
export { meshVolume, createMesherCache } from './mesher'
export type { VoxelMesh, MesherCache } from './mesher'
export { buildBlockGeometry } from './geometry'
export { parseBlockState, stripNamespace } from './model'
export { sortFacesByDepth, SORT_FACE_LIMIT } from './sort'
export { FOV, MIN_RADIUS, clamp, orbitPosition, panTarget } from './orbit'
export type { Vec3 } from './orbit'
export { DIRECTIONS } from './types'
export type {
  BlockGeometry,
  FaceDirection,
  MeshData,
  MeshFace,
  PackData,
  TextureRect,
  VoxelVolume,
} from './types'
