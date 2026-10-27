/**
 * 体素数据 → 可渲染结构
 *
 * 把后端的方块索引数组转成渲染库的 Structure。整份体积逐个建块会产生
 * 上百万个对象，这里先做一次遮挡剔除：六个邻居都不透明的实心方块谁也看不见，
 * 直接丢掉；液体额外按"同种液体也算实心"处理，只保留表层。
 */

import { BlockState, Structure, type BlockPos, type Resources } from '@mattzh72/lodestone'

export interface VoxelVolume {
  blocks: Uint16Array
  palette: readonly string[]
  size: [number, number, number]
  airIndex: number
}

export interface BuildStructureResult {
  structure: Structure
  /** 实际塞进结构的方块数（遮挡剔除后） */
  kept: number
  /** 原始非空气方块数 */
  total: number
}

const NEIGHBOURS: ReadonlyArray<readonly [number, number, number]> = [
  [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1],
];

/** 建结构并做遮挡剔除 */
export function buildStructure(volume: VoxelVolume, resources: Resources): BuildStructureResult {
  const { blocks, palette, size, airIndex } = volume
  const [sx, sy, sz] = size
  const layerSize = sx * sz

  const states: BlockState[] = new Array(palette.length)
  for (let i = 0; i < palette.length; i++) {
    states[i] = BlockState.parse(palette[i] ?? 'minecraft:air')
  }

  // 每种调色板条目只查一次不透明度；液体记为实心参与剔除，避免整片水都留下
  const cullMask = new Uint8Array(palette.length)
  for (let i = 0; i < palette.length; i++) {
    const state = states[i]
    const flags = resources.getBlockFlags(state.getName())
    const fluid = state.isFluid()
    cullMask[i] = flags?.opaque === true ? 2 : fluid ? 1 : 0
  }

  const at = (x: number, y: number, z: number): number =>
    x + z * sx + y * layerSize

  const stored: { pos: BlockPos; state: number }[] = []
  let total = 0

  for (let y = 0; y < sy; y++) {
    for (let z = 0; z < sz; z++) {
      for (let x = 0; x < sx; x++) {
        const id = blocks[at(x, y, z)]
        if (id === airIndex) continue
        total += 1;
        const mask = cullMask[id] ?? 0
        if (mask !== 0) {
          let hidden = true;
          for (const [dx, dy, dz] of NEIGHBOURS) {
            const nx = x + dx
            const ny = y + dy
            const nz = z + dz
            // 越界视为空气，边界方块的面必须保留
            if (nx < 0 || ny < 0 || nz < 0 || nx >= sx || ny >= sy || nz >= sz) {
              hidden = false;
              break;
            }
            const neighbour = blocks[at(nx, ny, nz)]
            if (neighbour === airIndex) {
              hidden = false;
              break;
            }
            const neighbourMask = cullMask[neighbour] ?? 0
            // 不透明方块之间互相遮挡；液体只和同种液体互相遮挡
            const blocksView = neighbourMask === 2 || (mask === 1 && neighbourMask === 1 && neighbour === id);
            if (!blocksView) {
              hidden = false;
              break;
            }
          }
          if (hidden) continue;
        }
        stored.push({ pos: [x, y, z], state: id });
      }
    }
  }

  return {
    structure: new Structure([sx, sy, sz], states, stored),
    kept: stored.length,
    total,
  };
}
