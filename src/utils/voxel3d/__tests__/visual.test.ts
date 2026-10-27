import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeAll, describe, expect, it } from 'vitest'
import { meshVolume } from '../mesher'
import { decodePng } from './png'
import type { PackData, VoxelVolume } from '../types'

type Vec3 = [number, number, number]

const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
const cross = (a: Vec3, b: Vec3): Vec3 => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
]
const dot = (a: Vec3, b: Vec3): number => a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
const unit = (a: Vec3): Vec3 => {
  const l = Math.hypot(a[0], a[1], a[2]) || 1
  return [a[0] / l, a[1] / l, a[2] / l]
};

/** 极简软件光栅化：只为验证朝向，不做插值抗锯齿 */
function render(
  mesh: { positions: Float32Array; normals: Float32Array; uvs: Float32Array; colors: Float32Array; indices: Uint32Array },
  atlas: { data: Uint8Array; width: number; height: number },
  eye: Vec3,
  target: Vec3,
  size: number,
): Uint8Array {
  const W = size
  const H = size
  const out = new Uint8Array(W * H * 4)
  const depth = new Float32Array(W * H).fill(Infinity)
  for (let i = 0; i < W * H; i++) {
    out[i * 4] = 18; out[i * 4 + 1] = 22; out[i * 4 + 2] = 31; out[i * 4 + 3] = 255;
  }
  const fwd = unit(sub(target, eye))
  // 正俯视时视线与世界上方向平行，叉积退化，换一个参考轴
  let rightRaw = cross(fwd, [0, 1, 0])
  if (Math.hypot(rightRaw[0], rightRaw[1], rightRaw[2]) < 1e-6) rightRaw = [1, 0, 0]
  const right = unit(rightRaw)
  const up = cross(right, fwd)
  const tanHalf = Math.tan((70 * Math.PI) / 360)
  const light = unit([0.6, 1, 0.35])
  const project = (v: Vec3) => {
    const rel = sub(v, eye)
    const z = dot(rel, fwd)
    if (z <= 0.001) return null
    return {
      x: (dot(rel, right) / (z * tanHalf) * 0.5 + 0.5) * W,
      y: (0.5 - (dot(rel, up) / (z * tanHalf)) * 0.5) * H,
      z,
    };
  };
  for (let t = 0; t < mesh.indices.length; t += 3) {
    const ia = mesh.indices[t]; const ib = mesh.indices[t + 1]; const ic = mesh.indices[t + 2]
    const pos = (i: number): Vec3 => [mesh.positions[i * 3], mesh.positions[i * 3 + 1], mesh.positions[i * 3 + 2]]
    const a = project(pos(ia)); const b = project(pos(ib)); const c = project(pos(ic))
    if (!a || !b || !c) continue
    const area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)
    if (Math.abs(area) < 1e-6) continue
    const sign = area > 0 ? 1 : -1
    const minX = Math.max(0, Math.floor(Math.min(a.x, b.x, c.x)))
    const maxX = Math.min(W - 1, Math.ceil(Math.max(a.x, b.x, c.x)))
    const minY = Math.max(0, Math.floor(Math.min(a.y, b.y, c.y)))
    const maxY = Math.min(H - 1, Math.ceil(Math.max(a.y, b.y, c.y)))
    for (let py = minY; py <= maxY; py++) {
      for (let px = minX; px <= maxX; px++) {
        const cx = px + 0.5; const cy = py + 0.5
        let w0 = ((b.x - a.x) * (cy - a.y) - (cx - a.x) * (b.y - a.y)) * sign
        let w1 = ((cx - a.x) * (c.y - a.y) - (c.x - a.x) * (cy - a.y)) * sign
        const total = Math.abs(area)
        w0 /= total; w1 /= total
        const w2 = 1 - w0 - w1
        if (w0 < 0 || w1 < 0 || w2 < 0) continue
        const z = a.z * w2 + b.z * w1 + c.z * w0
        const idx = py * W + px
        if (z >= depth[idx]) continue
        const u = mesh.uvs[ia * 2] * w2 + mesh.uvs[ib * 2] * w1 + mesh.uvs[ic * 2] * w0
        const v = mesh.uvs[ia * 2 + 1] * w2 + mesh.uvs[ib * 2 + 1] * w1 + mesh.uvs[ic * 2 + 1] * w0
        const tx = Math.min(atlas.width - 1, Math.max(0, Math.floor(u * atlas.width)))
        const ty = Math.min(atlas.height - 1, Math.max(0, Math.floor(v * atlas.height)))
        const o = (ty * atlas.width + tx) * 4
        if (atlas.data[o + 3] < 128) continue
        const tr = mesh.colors[ia * 3] * w2 + mesh.colors[ib * 3] * w1 + mesh.colors[ic * 3] * w0
        const tg = mesh.colors[ia * 3 + 1] * w2 + mesh.colors[ib * 3 + 1] * w1 + mesh.colors[ic * 3 + 1] * w0
        const tb = mesh.colors[ia * 3 + 2] * w2 + mesh.colors[ib * 3 + 2] * w1 + mesh.colors[ic * 3 + 2] * w0
        const n = unit([
          mesh.normals[ia * 3] * w2 + mesh.normals[ib * 3] * w1 + mesh.normals[ic * 3] * w0,
          mesh.normals[ia * 3 + 1] * w2 + mesh.normals[ib * 3 + 1] * w1 + mesh.normals[ic * 3 + 1] * w0,
          mesh.normals[ia * 3 + 2] * w2 + mesh.normals[ib * 3 + 2] * w1 + mesh.normals[ic * 3 + 2] * w0,
        ])
        const shade = 0.55 + 0.45 * Math.max(0, dot(n, light))
        depth[idx] = z
        out[idx * 4] = Math.min(255, atlas.data[o] * tr * shade)
        out[idx * 4 + 1] = Math.min(255, atlas.data[o + 1] * tg * shade)
        out[idx * 4 + 2] = Math.min(255, atlas.data[o + 2] * tb * shade)
        out[idx * 4 + 3] = 255
      }
    }
  }
  return out
}

let pack: PackData
let atlas: { data: Uint8Array; width: number; height: number }

beforeAll(() => {
  const dir = resolve(process.cwd(), 'node_modules/@mattzh72/lodestone/assets/default-pack')
  const raw = JSON.parse(readFileSync(resolve(dir, 'assets.json'), 'utf8')) as {
    blockstates: Record<string, unknown>
    models: Record<string, unknown>
    textures: Record<string, [number, number, number, number]>
  }
  const decoded = decodePng(readFileSync(resolve(dir, 'atlas.png')))
  atlas = { data: decoded.data, width: decoded.width, height: decoded.height }
  pack = {
    blockstates: raw.blockstates,
    models: raw.models,
    textures: raw.textures,
    atlasSize: [atlas.width, atlas.height],
    opaqueBlocks: new Set(['stone', 'grass_block', 'dirt', 'oak_planks']),
    transparentBlocks: new Set(['glass', 'oak_leaves']),
  }
});

/** 造一个 N×N 的草地，上面立一根石头柱 */
function grassField(size: number): VoxelVolume {
  const blocks = new Uint16Array(size * size * 4)
  const sy = 4
  for (let z = 0; z < size; z++) {
    for (let x = 0; x < size; x++) {
      blocks[x + z * size + 0 * size * size] = 1
    }
  }
  return {
    blocks,
    palette: ['minecraft:air', 'minecraft:grass_block[snowy=false]'],
    size: [size, sy, size],
    airIndex: 0,
  }
}

const BG: [number, number, number] = [18, 22, 31]

function isBackground(px: Uint8Array, offset: number): boolean {
  return px[offset] === BG[0] && px[offset + 1] === BG[1] && px[offset + 2] === BG[2]
}

/** 画面上实际有几何的矩形范围，取样时用它对齐，避免写死坐标 */
function objectBounds(px: Uint8Array, size: number) {
  let minX = size; let minY = size; let maxX = -1; let maxY = -1
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      if (isBackground(px, (y * size + x) * 4)) continue
      if (x < minX) minX = x
      if (x > maxX) maxX = x
      if (y < minY) minY = y
      if (y > maxY) maxY = y
    }
  }
  return { minX, minY, maxX, maxY, empty: maxX < 0 }
}

function averageColor(px: Uint8Array, size: number, x0: number, y0: number, x1: number, y1: number) {
  let r = 0; let g = 0; let b = 0; let n = 0
  const xs = Math.max(0, Math.floor(x0))
  const xe = Math.min(size, Math.ceil(x1))
  const ys = Math.max(0, Math.floor(y0))
  const ye = Math.min(size, Math.ceil(y1))
  for (let y = ys; y < ye; y++) {
    for (let x = xs; x < xe; x++) {
      const o = (y * size + x) * 4
      if (isBackground(px, o)) continue
      r += px[o]; g += px[o + 1]; b += px[o + 2]; n++
    }
  }
  return n ? { r: r / n, g: g / n, b: b / n, count: n } : { r: 0, g: 0, b: 0, count: 0 }
}

describe('rendered output', () => {
  it('should keep every triangle winding consistent with its normal', () => {
    const mesh = meshVolume(grassField(4), pack)
    for (const part of [mesh.opaque, mesh.translucent]) {
      for (let t = 0; t < part.indices.length; t += 3) {
        const get = (i: number): Vec3 => [
          part.positions[i * 3], part.positions[i * 3 + 1], part.positions[i * 3 + 2],
        ]
        const a = get(part.indices[t])
        const b = get(part.indices[t + 1])
        const c = get(part.indices[t + 2])
        const faceNormal = unit(cross(sub(b, a), sub(c, a)))
        const stated: Vec3 = [
          part.normals[part.indices[t] * 3],
          part.normals[part.indices[t] * 3 + 1],
          part.normals[part.indices[t] * 3 + 2],
        ]
        // 叉积必须与声明的法线同向，否则面朝里、会被背面剔除掉
        expect(dot(faceNormal, stated)).toBeGreaterThan(0.9)
      }
    }
  });

  it('should show green grass when looking straight down', () => {
    const mesh = meshVolume(grassField(6), pack)
    const px = render(mesh.opaque, atlas, [3.5, 8, 3.5], [3.5, 0, 3.5], 96)
    const box = objectBounds(px, 96)
    expect(box.empty).toBe(false)
    const midX = (box.minX + box.maxX) / 2
    const midY = (box.minY + box.maxY) / 2
    const span = (box.maxX - box.minX) * 0.2
    const center = averageColor(px, 96, midX - span, midY - span, midX + span, midY + span)
    expect(center.count).toBeGreaterThan(300)
    // 草方块顶面是灰度贴图乘生物群系色，渲染出来必须是绿大于蓝
    expect(center.g).toBeGreaterThan(center.b)
    expect(center.g).toBeGreaterThan(center.r)
  });

  it('should draw the grass overlay band at the top of a side face', () => {
    const mesh = meshVolume(grassField(6), pack)
    // 正对南侧看，草侧面应是上半偏绿、下半偏棕
    // 相机贴着草地方块的高度，正对南面
    const px = render(mesh.opaque, atlas, [3.5, 0.5, 12], [3.5, 0.5, 3.5], 256)
    const box = objectBounds(px, 256)
    expect(box.empty).toBe(false)
    const height = box.maxY - box.minY
    const midX = (box.minX + box.maxX) / 2
    const halfW = (box.maxX - box.minX) * 0.15
    // 方块侧面被斜视角压扁，取最靠近顶边的一条作为草带，靠近底边的一条作为泥土
    const upper = averageColor(px, 256, midX - halfW, box.minY + height * 0.02, midX + halfW, box.minY + height * 0.10)
    const lower = averageColor(px, 256, midX - halfW, box.maxY - height * 0.10, midX + halfW, box.maxY - height * 0.02)
    expect(upper.count).toBeGreaterThan(50)
    expect(lower.count).toBeGreaterThan(50)
    // 上半是草：绿色分量明显高于蓝色；下半是泥土：蓝分量低且整体偏暗
    expect(upper.g - upper.b).toBeGreaterThan(lower.g - lower.b)
  });

  it('should render stone as a neutral gray', () => {
    const blocks = new Uint16Array(2 * 2 * 2)
    blocks[0] = 1
    const mesh = meshVolume({
      blocks,
      palette: ['minecraft:air', 'minecraft:stone'],
      size: [2, 2, 2],
      airIndex: 0,
    }, pack)
    const px = render(mesh.opaque, atlas, [0.5, 0.5, 4], [0.5, 0.5, 0], 128)
    const box = objectBounds(px, 128)
    expect(box.empty).toBe(false)
    const midX = (box.minX + box.maxX) / 2
    const midY = (box.minY + box.maxY) / 2
    const span = (box.maxX - box.minX) * 0.2
    const center = averageColor(px, 128, midX - span, midY - span, midX + span, midY + span)
    expect(center.count).toBeGreaterThan(50)
    // 石头没有 tint，三个通道应当接近
    expect(Math.abs(center.r - center.g)).toBeLessThan(8)
    expect(Math.abs(center.g - center.b)).toBeLessThan(8)
  });
});
