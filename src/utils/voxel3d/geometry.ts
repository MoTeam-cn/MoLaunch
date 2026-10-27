/**
 * 模型几何生成
 *
 * 把解析后的元素拆成带 UV 的四边形。顶点绕序统一为从外侧看逆时针，
 * 每个面按「左上、左下、右下、右上」的顺序对应 uv 的四个角。
 */
import { parseBlockState, resolveModel, resolveTextureRef, selectVariant } from './model'
import { resolveTint } from './tints'
import type { BlockGeometry, FaceDirection, MeshFace, PackData, TextureRect } from './types'

type Vec3 = [number, number, number]

/**
 * 六个面的顶点顺序（逆时针朝外）与对应的 uv 角序
 *
 * uv 的四个角按 [左上, 左下, 右下, 右上] 编号，uvOrder 表示每个顶点各取哪个角。
 * 判断「哪个顶点是左上」必须站在该面外侧看：屏幕上的「右」对 north 是 -x、
 * 对 south 是 +x、对 up 是 +x 且「上」是 -z、对 down 是 +x 且「上」是 +z。
 */
const FACE_LAYOUT: Record<FaceDirection, { corners: [Vec3, Vec3, Vec3, Vec3]; uvOrder: [number, number, number, number]; normal: Vec3 }> = {
  down: {
    corners: [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]],
    uvOrder: [0, 3, 2, 1],
    normal: [0, -1, 0],
  },
  up: {
    corners: [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]],
    uvOrder: [0, 1, 2, 3],
    normal: [0, 1, 0],
  },
  north: {
    corners: [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]],
    uvOrder: [2, 3, 0, 1],
    normal: [0, 0, -1],
  },
  south: {
    corners: [[1, 0, 1], [1, 1, 1], [0, 1, 1], [0, 0, 1]],
    uvOrder: [2, 3, 0, 1],
    normal: [0, 0, 1],
  },
  west: {
    corners: [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]],
    uvOrder: [1, 0, 3, 2],
    normal: [-1, 0, 0],
  },
  east: {
    corners: [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]],
    uvOrder: [1, 0, 3, 2],
    normal: [1, 0, 0],
  },
}

/** 绕 Y 轴旋转 90 度的倍数（blockstate 的 y） */
function rotateY(point: Vec3, degrees: number): Vec3 {
  const steps = ((Math.round(degrees / 90) % 4) + 4) % 4
  let x = point[0]
  let z = point[2]
  for (let i = 0; i < steps; i++) {
    const nx = z
    const nz = 1 - x
    x = nx
    z = nz
  }
  return [x, point[1], z]
}

/** 绕 X 轴旋转 90 度的倍数（blockstate 的 x） */
function rotateX(point: Vec3, degrees: number): Vec3 {
  const steps = ((Math.round(degrees / 90) % 4) + 4) % 4
  let y = point[1]
  let z = point[2]
  for (let i = 0; i < steps; i++) {
    const ny = z
    const nz = 1 - y
    y = ny
    z = nz
  }
  return [point[0], y, z]
}

/** 面 UV 的四个角，按 uvOrder 取到实际坐标 */
function faceUv(rect: TextureRect, uv: [number, number, number, number], atlas: [number, number]) {
  const [ax, ay, aw, ah] = rect
  const [aw2, ah2] = atlas
  const corners: [number, number][] = [
    [(ax + (uv[0] / 16) * aw) / aw2, (ay + (uv[1] / 16) * ah) / ah2],
    [(ax + (uv[0] / 16) * aw) / aw2, (ay + (uv[3] / 16) * ah) / ah2],
    [(ax + (uv[2] / 16) * aw) / aw2, (ay + (uv[3] / 16) * ah) / ah2],
    [(ax + (uv[2] / 16) * aw) / aw2, (ay + (uv[1] / 16) * ah) / ah2],
  ]
  return corners
}

/** 多元素模型每层的外扩比例，单元素模型不扩 */
function elementSpread(elementCount: number): number {
  return elementCount > 1 ? 0.0012 : 0
}

/** 生成单个方块的全部面；结果按方块状态缓存 */
export function buildBlockGeometry(
  state: string,
  pack: PackData,
  cache: Map<string, BlockGeometry>,
): BlockGeometry {
  const cached = cache.get(state)
  if (cached) return cached

  const empty: BlockGeometry = { faces: [], translucent: false }
  const { name, props } = parseBlockState(state)
  const variant = selectVariant(pack.blockstates[name], props)
  if (!variant) {
    cache.set(state, empty)
    return empty
  }
  const model = resolveModel(variant.model, pack.models)
  if (!model || !model.elements) {
    cache.set(state, empty)
    return empty
  }

  const textureVars = model.textures ?? {}
  const faces: MeshFace[] = []
  const rotX = variant.x ?? 0
  const rotY = variant.y ?? 0

  // 草方块这类模型有两个完全共面的元素（本体 + 侧面覆盖层），
  // 深度相同时会互相啃出斑驳，按元素顺序把后面的略微外扩千分之一
  const spread = 1 + elementSpread(model.elements.length)

  for (const [elementIndex, element] of model.elements.entries()) {
    const from = element.from ?? [0, 0, 0]
    const to = element.to ?? [16, 16, 16]
    const factor = 1 + elementIndex * (spread - 1)
    const lo: Vec3 = [from[0] / 16, from[1] / 16, from[2] / 16]
    const hi: Vec3 = [to[0] / 16, to[1] / 16, to[2] / 16]

    for (const [dir, face] of Object.entries(element.faces ?? {}) as [FaceDirection, NonNullable<typeof element.faces>[FaceDirection]][]) {
      if (!face) continue
      const layout = FACE_LAYOUT[dir]
      if (!layout) continue
      const textureId = face.texture ? resolveTextureRef(face.texture, textureVars) : null
      if (!textureId) continue
      const rect = pack.textures[textureId]
      if (!rect) continue

      const uvRaw: [number, number, number, number] = face.uv ?? [0, 0, 16, 16]
      const uvCorners = faceUv(rect, uvRaw, pack.atlasSize)
      const vertices = layout.corners.map((corner) => {
        const point: Vec3 = [
          corner[0] === 0 ? lo[0] : hi[0],
          corner[1] === 0 ? lo[1] : hi[1],
          corner[2] === 0 ? lo[2] : hi[2],
        ]
        // 从方块中心向外微扩，保证后出现的元素压在前一层之上
        const pushed: Vec3 = [
          0.5 + (point[0] - 0.5) * factor,
          0.5 + (point[1] - 0.5) * factor,
          0.5 + (point[2] - 0.5) * factor,
        ]
        let rotated = rotateX(pushed, rotX)
        rotated = rotateY(rotated, rotY)
        return rotated
      })

      // 按顶点顺序取角，四个角不一定构成矩形对角，所以逐个保留
      const uvCornersForVertices = layout.uvOrder.map((idx) => uvCorners[idx])
      const us = uvCornersForVertices.map((c) => c[0])
      const vs = uvCornersForVertices.map((c) => c[1])
      const uv: [number, number, number, number] = [
        Math.min(...us),
        Math.min(...vs),
        Math.max(...us),
        Math.max(...vs),
      ]
      faces.push({
        vertices,
        uvCorners: uvCornersForVertices,
        uv,
        cullface: face.cullface,
        tint: resolveTint(name, face.tintindex),
        normal: layout.normal,
      })
    }
  }

  const result: BlockGeometry = { faces, translucent: pack.transparentBlocks.has(name) }
  cache.set(state, result)
  return result
}
