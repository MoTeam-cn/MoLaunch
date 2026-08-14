#!/usr/bin/env node
/**
 * 生成合成配方生成器内置数据资产（输出至 src/utils/recipe-generator/assets/）
 *
 * 数据源（均为 MIT 许可，与 Axolotl GPL 资产无关）：
 * - 物品表：PrismarineJS/minecraft-data（items.json / blocks.json，覆盖 1.12~26.1）
 * - 纹理：PrismarineJS/minecraft-assets（items_textures.json + items|blocks/*.png，覆盖 1.12~1.21.11）
 * - 标签：destruc7i0n/crafting（vanilla-tags，覆盖 1.14~26.2）
 * - 中文名：Mojang 官方 assets 的 zh_cn.json（取最新 26.2）
 *
 * 26.2 无独立物品表：以 26.1 为基础 + 26.2 标签中新增物品补齐，纹理回落 1.21.11。
 * 1.12 旧版：按 minecraft-data blocks.json variations 展开 data 变体，经
 * 自动匹配 + 静态表映射到现代扁平 ID 以复用现代纹理与中文名。
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { PNG } from 'pngjs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const CACHE_DIR = path.join(__dirname, '.cache')
const OUT_DIR = path.resolve(__dirname, '../../src/utils/recipe-generator/assets')

// ---------------------------------------------------------------- 版本映射

/** 工具支持的 Java 版本（顺序即显示顺序） */
export const OUR_VERSIONS = [
  '1.12', '1.13', '1.14', '1.15', '1.16', '1.17', '1.18', '1.19', '1.20',
  '1.21', '1.21.2', '1.21.4', '1.21.5', '1.21.6', '1.21.7', '1.21.9', '1.21.11', '26.1', '26.2',
]

/** 工具版本 -> minecraft-assets 数据目录版本（26.x 无对应目录，回落 1.21.11） */
const ASSET_VERSION = {
  '1.12': '1.12', '1.13': '1.13', '1.14': '1.14.4', '1.15': '1.15.2', '1.16': '1.16.1',
  '1.17': '1.17.1', '1.18': '1.18.1', '1.19': '1.19.1', '1.20': '1.20.2', '1.21': '1.21.1',
  '1.21.2': '1.21.4', '1.21.4': '1.21.4', '1.21.5': '1.21.5', '1.21.6': '1.21.6',
  '1.21.7': '1.21.7', '1.21.9': '1.21.9', '1.21.11': '1.21.11', '26.1': '1.21.11', '26.2': '1.21.11',
}

/** 工具版本 -> minecraft-data 实际含 items.json 的目录（部分小版本目录无物品表） */
const DATA_VERSION = {
  '1.12': '1.12', '1.13': '1.13', '1.14': '1.14.4', '1.15': '1.15.2', '1.16': '1.16.1',
  '1.17': '1.17', '1.18': '1.18', '1.19': '1.19', '1.20': '1.20.2', '1.21': '1.21.1',
  '1.21.2': '1.21.3', '1.21.4': '1.21.4', '1.21.5': '1.21.5', '1.21.6': '1.21.6',
  '1.21.7': '1.21.8', '1.21.9': '1.21.9', '1.21.11': '1.21.11', '26.1': '26.1', '26.2': '26.1',
}

const MC_DATA = 'https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/pc'
const MC_ASSETS = 'https://raw.githubusercontent.com/PrismarineJS/minecraft-assets/master/data'
const CRAFTING_TAGS = 'https://raw.githubusercontent.com/destruc7i0n/crafting/main/src/data/generated/vanilla-tags'

// ---------------------------------------------------------------- 1.12 旧版映射

/** 1.12 方块颜色顺序（与羊毛/染色玻璃一致，0=白色） */
const BLOCK_COLORS = [
  'white', 'orange', 'magenta', 'light_blue', 'yellow', 'lime', 'pink', 'gray',
  'light_gray', 'cyan', 'purple', 'blue', 'brown', 'green', 'red', 'black',
]

/** 1.12 变体（blocks.json metadata>0）-> 现代扁平 ID（自动匹配失败的场景） */
const LEGACY_VARIANT = {
  log: { 1: 'spruce_log', 2: 'birch_log', 3: 'jungle_log' },
  log2: { 1: 'dark_oak_log' },
  stone_slab: {
    0: 'smooth_stone_slab', 1: 'sandstone_slab', 2: 'oak_slab', 3: 'cobblestone_slab',
    4: 'brick_slab', 5: 'stone_brick_slab', 6: 'nether_brick_slab', 7: 'quartz_slab',
  },
  wooden_slab: {
    0: 'oak_slab', 1: 'spruce_slab', 2: 'birch_slab', 3: 'jungle_slab',
    4: 'acacia_slab', 5: 'dark_oak_slab',
  },
  monster_egg: { 1: 'infested_cobblestone', 2: 'infested_stone_bricks' },
  fish: { 1: 'salmon', 2: 'tropical_fish', 3: 'pufferfish' },
  cooked_fish: { 1: 'cooked_salmon' },
  stained_hardened_clay: Object.fromEntries(
    BLOCK_COLORS.slice(1).map((c, i) => [i + 1, `${c}_terracotta`]),
  ),
}

/** 1.12 物品（items.json）-> 现代扁平 ID（自动匹配失败的场景） */
const ITEM_RENAME = {
  grass: 'grass_block',
  stonebrick: 'stone_bricks',
  brick_block: 'bricks',
  waterlily: 'lily_pad',
  netherbrick: 'nether_brick',
  sign: 'oak_sign',
  bed: 'red_bed',
  boat: 'oak_boat',
  wooden_door: 'oak_door',
  wooden_trapdoor: 'oak_trapdoor',
  wooden_button: 'oak_button',
  wooden_pressure_plate: 'oak_pressure_plate',
  wooden_stairs: 'oak_stairs',
  wooden_fence: 'oak_fence',
  fence: 'oak_fence',
  fence_gate: 'oak_fence_gate',
  wooden_fence_gate: 'oak_fence_gate',
  wooden_slab: 'oak_slab',
  hardwood_stairs: 'spruce_stairs',
  magma: 'magma_block',
  skull: 'skeleton_skull',
  snow: 'snow_block',
  snow_layer: 'snow',
  end_bricks: 'end_stone_bricks',
  red_nether_brick: 'red_nether_bricks',
  wool: 'white_wool',
  log: 'oak_log',
  log2: 'acacia_log',
  stone_slab: 'smooth_stone_slab',
  monster_egg: 'infested_stone',
  hardened_clay: 'terracotta',
  stained_hardened_clay: 'white_terracotta',
  fish: 'cod',
  cooked_fish: 'cooked_cod',
  seeds: 'wheat_seeds',
  totem: 'totem_of_undying',
  web: 'cobweb',
  speckled_melon: 'glistering_melon_slice',
  empty_map: 'map',
  fireworks: 'firework_rocket',
  firework_charge: 'firework_star',
}

/** 1.12 染料（非方块变体，items.json 无 data 信息）-> 现代扁平 ID */
const DYE_VARIANTS = [
  [0, 'ink_sac'], [1, 'rose_red'], [2, 'cactus_green'], [3, 'cocoa_beans'], [4, 'lapis_lazuli'],
  [5, 'purple_dye'], [6, 'cyan_dye'], [7, 'light_gray_dye'], [8, 'gray_dye'], [9, 'pink_dye'],
  [10, 'lime_dye'], [11, 'dandelion_yellow'], [12, 'light_blue_dye'], [13, 'magenta_dye'],
  [14, 'orange_dye'], [15, 'bone_meal'],
]

/** 1.12 旗帜（物品 data 0-15 = 颜色）-> 现代 <color>_banner */
const BANNER_VARIANTS = BLOCK_COLORS

// ---------------------------------------------------------------- 网络辅助

async function fetchRetry(url, attempts = 4) {
  let lastErr
  for (let i = 0; i < attempts; i += 1) {
    try {
      const res = await fetch(url, { redirect: 'follow' })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      return await res.arrayBuffer()
    } catch (err) {
      lastErr = err
      await new Promise((r) => setTimeout(r, 800 * (i + 1)))
    }
  }
  throw new Error(`下载失败 ${url}: ${lastErr?.message ?? '未知错误'}`)
}

async function fetchJson(url) {
  const buf = await fetchRetry(url)
  return JSON.parse(Buffer.from(buf).toString('utf-8'))
}

/** 下载纹理 PNG（带本地缓存，可断点续跑） */
async function fetchTexturePng(assetVersion, key) {
  const cacheFile = path.join(CACHE_DIR, 'png', `${assetVersion}_${key.replaceAll('/', '_')}`)
  if (fs.existsSync(cacheFile)) return fs.readFileSync(cacheFile)
  const url = `${MC_ASSETS}/${assetVersion}/${key}.png`
  const buf = Buffer.from(await fetchRetry(url, 2))
  if (buf.length === 0) return null
  fs.mkdirSync(path.dirname(cacheFile), { recursive: true })
  fs.writeFileSync(cacheFile, buf)
  return buf
}

// ---------------------------------------------------------------- 工具函数

const norm = (s) => (s ?? '').toLowerCase().replace(/[^a-z0-9]+/g, '')
const normSoft = (s) => norm(s).replace(/wood/g, '')

/** 纹理路径规范化（minecraft:block/stone -> blocks/stone；items/iron_ingot -> items/iron_ingot） */
function normalizeTextureKey(raw) {
  if (!raw) return null
  let p = String(raw).replace(/^minecraft:/, '')
  if (p.startsWith('block/')) return `blocks/${p.slice('block/'.length)}`
  if (p.startsWith('blocks/')) return p
  if (p.startsWith('item/')) return `items/${p.slice('item/'.length)}`
  if (p.startsWith('items/')) return p
  if (p === 'missingno') return null
  return p
}

// ---------------------------------------------------------------- 主流程

async function main() {
  fs.mkdirSync(CACHE_DIR, { recursive: true })
  fs.mkdirSync(OUT_DIR, { recursive: true })
  fs.mkdirSync(path.join(OUT_DIR, 'items'), { recursive: true })
  fs.mkdirSync(path.join(OUT_DIR, 'tags'), { recursive: true })

  // 1. 现代参考物品表（26.1）用于自动匹配 + 英文名兜底
  const modernItems = await fetchJson(`${MC_DATA}/26.1/items.json`)
  const modernById = new Map(modernItems.map((i) => [i.name, i]))
  const modernByNameNorm = new Map()
  const modernByIdNorm = new Map()
  for (const item of modernItems) {
    modernByNameNorm.set(norm(item.displayName), item.name)
    modernByIdNorm.set(norm(item.name), item.name)
  }

  // 2. 各版本 items_textures（name -> 纹理 key）
  const assetsVersions = [...new Set(Object.values(ASSET_VERSION))]
  const texturesByAsset = new Map()
  for (const assetVer of assetsVersions) {
    const list = await fetchJson(`${MC_ASSETS}/${assetVer}/items_textures.json`)
    const map = new Map()
    for (const entry of list) {
      const key = normalizeTextureKey(entry.texture)
      if (key) map.set(entry.name, key)
    }
    texturesByAsset.set(assetVer, map)
  }
  const latestTextures = texturesByAsset.get('1.21.11')
  console.log('[textures] 已加载资产版本:', assetsVersions.join(', '))

  // 3. 中文名（26.2 官方 zh_cn.json）
  const zhMap = await fetchZhCn()
  console.log('[zh_cn] 已加载 26.2 官方中文名,', zhMap.size, '条')

  const zhOf = (modernId) =>
    zhMap.get(`block.minecraft.${modernId}`) ?? zhMap.get(`item.minecraft.${modernId}`)

  // 4. 逐版本生成物品表
  const versionManifests = new Map()
  for (const version of OUR_VERSIONS) {
    console.log(`[items] 生成 ${version} ...`)
    const entries = await buildVersionItems(version, {
      modernById, modernByNameNorm, modernByIdNorm, latestTextures, zhOf,
      texturesByAsset,
    })
    const manifest = { version, items: entries }
    versionManifests.set(version, manifest)
    fs.writeFileSync(
      path.join(OUT_DIR, 'items', `${version}.json`),
      JSON.stringify(manifest),
    )
  }

  // 5. 标签（1.14+；1.12/1.13 不支持标签 -> 空对象）
  for (const version of OUR_VERSIONS) {
    let tags = {}
    if (version !== '1.12' && version !== '1.13') {
      try {
        tags = await fetchJson(`${CRAFTING_TAGS}/${version}.json`)
      } catch (err) {
        console.warn(`[tags] ${version} 下载失败, 使用空标签: ${err.message}`)
      }
    }
    fs.writeFileSync(path.join(OUT_DIR, 'tags', `${version}.json`), JSON.stringify(tags))
  }
  console.log('[tags] 完成')

  // 6. 纹理图集
  const { atlasPng, layout, usedCount, atlasSize } = await buildAtlas(versionManifests)
  fs.writeFileSync(path.join(OUT_DIR, 'texture-atlas.png'), PNG.sync.write(atlasPng))
  fs.writeFileSync(
    path.join(OUT_DIR, 'texture-atlas.json'),
    JSON.stringify({ size: atlasSize, layout }),
  )
  fs.writeFileSync(
    path.join(OUT_DIR, 'sources.json'),
    JSON.stringify(
      {
        comment: 'MIT 许可数据源，非 Axolotl GPL 资产。',
        items: 'PrismarineJS/minecraft-data (MIT)',
        textures: 'PrismarineJS/minecraft-assets (MIT)',
        tags: 'destruc7i0n/crafting (MIT)',
        names: 'Mojang official assets zh_cn.json',
        atlasSize: [atlasPng.width, atlasPng.height],
      },
      null,
      2,
    ),
  )
  console.log(
    `[atlas] 完成: ${usedCount} 个纹理 -> ${atlasPng.width}x${atlasPng.height}`,
  )
}

/** 构建单个版本的物品条目列表 */
async function buildVersionItems(version, ctx) {
  if (version === '1.12') return buildLegacyItems(ctx)
  const { latestTextures, zhOf, texturesByAsset } = ctx
  const dataVersion = DATA_VERSION[version] ?? version
  const items = await fetchJson(`${MC_DATA}/${dataVersion}/items.json`)
  const assetVersion = ASSET_VERSION[version]
  const textureMap = texturesByAsset.get(assetVersion)

  const entries = []
  const seen = new Set()
  for (const item of items) {
    if (seen.has(item.name)) continue
    seen.add(item.name)
    const textureKey = textureMap.get(item.name) ?? latestTextures.get(item.name) ?? null
    entries.push({
      id: `minecraft:${item.name}`,
      name: item.displayName,
      zh: zhOf(item.name) ?? '',
      texture: textureKey,
    })
  }

  // 26.2：以 26.2 官方标签中新增的物品补齐（标签数据已含 26.2 全量物品引用）
  if (version === '26.2') {
    const tags = await fetchJson(`${CRAFTING_TAGS}/26.2.json`)
    const known = new Set(items.map((i) => i.name))
    const added = new Set()
    for (const ids of Object.values(tags)) {
      for (const fullId of ids) {
        const name = fullId.split(':')[1] ?? fullId
        if (known.has(name) || added.has(name) || seen.has(name)) continue
        added.add(name)
        entries.push({
          id: `minecraft:${name}`,
          name: readableFromId(name),
          zh: zhOf(name) ?? '',
          texture: latestTextures.get(name) ?? null,
        })
      }
    }
    if (added.size > 0) console.log(`[26.2] 标签补充新物品 ${added.size} 个`)
  }
  return entries
}

/** 1.12 旧版：items + blocks variations 展开 + 变体映射 */
async function buildLegacyItems(ctx) {
  const { modernById, modernByNameNorm, modernByIdNorm, latestTextures, zhOf } = ctx
  const items = await fetchJson(`${MC_DATA}/1.12/items.json`)
  const blocks = await fetchJson(`${MC_DATA}/1.12/blocks.json`)
  const itemNames = new Set(items.map((i) => i.name))

  const resolveModern = (displayName, legacyName, metadata) => {
    if (legacyName === 'dye') return DYE_VARIANTS[metadata]?.[1] ?? null
    if (legacyName === 'banner') return `${BANNER_VARIANTS[metadata] ?? 'white'}_banner`
    if (/^record_/.test(legacyName)) return legacyName.replace('record_', 'music_disc_')
    if (metadata !== undefined && metadata > 0) {
      const table = LEGACY_VARIANT[legacyName]
      if (table?.[metadata]) return table[metadata]
    }
    const renamed = ITEM_RENAME[legacyName]
    if (renamed) return renamed
    const auto =
      modernByNameNorm.get(normSoft(displayName)) ?? modernByIdNorm.get(normSoft(displayName))
    if (auto) return auto
    return modernByIdNorm.get(norm(legacyName)) ?? null
  }

  const pushEntry = (entries, seen, id, displayName, modernId) => {
    if (!modernId) return
    if (seen.has(id)) return
    seen.add(id)
    const modern = modernById.get(modernId)
    const textureKey = latestTextures.get(modernId) ?? null
    entries.push({
      id,
      name: displayName,
      zh: zhOf(modernId) ?? '',
      texture: textureKey,
    })
  }

  const entries = []
  const seen = new Set()

  // 基础条目（items.json）
  for (const item of items) {
    if (item.name === 'dye' || item.name === 'spawn_egg') continue
    const modernId = resolveModern(item.displayName, item.name, 0)
    pushEntry(entries, seen, `minecraft:${item.name}`, item.displayName, modernId)
  }

  // 方块变体展开（blocks.json variations, 仅 metadata>0）
  for (const block of blocks) {
    if (!itemNames.has(block.name)) continue
    const variations = block.variations ?? []
    const seenModern = new Set()
    for (const v of variations) {
      if (!v.metadata || v.metadata <= 0) continue
      const modernId = resolveModern(v.displayName, block.name, v.metadata)
      if (!modernId || seenModern.has(modernId)) continue
      seenModern.add(modernId)
      pushEntry(entries, seen, `minecraft:${block.name}:${v.metadata}`, v.displayName, modernId)
    }
  }

  // 染料 16 色
  for (const [data, modernId] of DYE_VARIANTS) {
    const modern = modernById.get(modernId)
    pushEntry(
      entries, seen, `minecraft:dye:${data}`, modern?.displayName ?? modernId, modernId,
    )
  }

  // 旗帜 16 色
  for (const [data, color] of BANNER_VARIANTS.entries()) {
    const modernId = `${color}_banner`
    const modern = modernById.get(modernId)
    pushEntry(
      entries, seen, `minecraft:banner:${data}`, modern?.displayName ?? `${color} banner`, modernId,
    )
  }

  return entries
}

function readableFromId(id) {
  return id
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

/** 下载最新版官方 zh_cn.json（走 Mojang version manifest -> assetIndex） */
async function fetchZhCn() {
  const cacheFile = path.join(CACHE_DIR, 'zh_cn_26.2.json')
  if (fs.existsSync(cacheFile)) return new Map(Object.entries(JSON.parse(fs.readFileSync(cacheFile, 'utf-8'))))

  const manifest = await fetchJson('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json')
  const verMeta = manifest.versions.find((v) => v.id === '26.2')
  if (!verMeta) throw new Error('未找到 26.2 版本元数据')
  const verJson = await fetchJson(verMeta.url)
  const assetIndex = await fetchJson(verJson.assetIndex.url)
  const langObj = assetIndex.objects['minecraft/lang/zh_cn.json']
  if (!langObj) throw new Error('26.2 资产索引缺少 zh_cn.json')
  const hash = langObj.hash
  const url = `https://resources.download.minecraft.net/${hash.slice(0, 2)}/${hash}`
  const buf = Buffer.from(await fetchRetry(url))
  const obj = JSON.parse(buf.toString('utf-8'))
  fs.writeFileSync(cacheFile, JSON.stringify(obj))
  return new Map(Object.entries(obj))
}

/** 收集所有版本用到的纹理 -> 下载 -> 打包图集 */
async function buildAtlas(versionManifests) {
  // 每个纹理 key -> 引用它的最高资产版本
  const textureRefs = new Map()
  for (const manifest of versionManifests.values()) {
    const assetVer = ASSET_VERSION[manifest.version]
    for (const item of manifest.items) {
      if (!item.texture) continue
      const prev = textureRefs.get(item.texture)
      if (!prev || rankVersion(assetVer) > rankVersion(prev)) {
        textureRefs.set(item.texture, assetVer)
      }
    }
  }

  const TILE_PAD = 0
  const tiles = []
  const failed = []
  let index = 0
  for (const [key, assetVer] of textureRefs) {
    index += 1
    const buf = await fetchTexturePng(assetVer, key)
    if (!buf) {
      failed.push(key)
      continue
    }
    let png
    try {
      png = PNG.sync.read(buf)
    } catch {
      failed.push(key)
      continue
    }
    tiles.push({ key, png })
    if (index % 200 === 0) console.log(`[atlas] 已下载 ${index}/${textureRefs.size}`)
  }

  // 行打包（宽度 2048，保持 16 对齐的简单货架算法）
  const ATLAS_W = 2048
  const layout = {}
  let x = 0
  let y = 0
  let rowH = 0
  const atlas = new PNG({ width: ATLAS_W, height: 2048 })
  for (const tile of tiles) {
    const { width: w, height: h } = tile.png
    if (x + w > ATLAS_W) {
      x = 0
      y += rowH + TILE_PAD
      rowH = 0
    }
    PNG.bitblt(tile.png, atlas, 0, 0, w, h, x, y)
    layout[tile.key] = [x, y, w, h]
    x += w + TILE_PAD
    rowH = Math.max(rowH, h)
  }
  const atlasH = y + rowH
  const final = new PNG({ width: ATLAS_W, height: atlasH })
  PNG.bitblt(atlas, final, 0, 0, ATLAS_W, atlasH, 0, 0)
  console.log(`[atlas] 纹理下载失败 ${failed.length} 个: ${failed.slice(0, 10).join(', ')}`)
  return { atlasPng: final, layout, usedCount: tiles.length, atlasSize: [ATLAS_W, atlasH] }
}

function rankVersion(v) {
  const parts = v.split('.').map((n) => Number(n) || 0)
  return parts[0] * 10000 + (parts[1] ?? 0) * 100 + (parts[2] ?? 0)
}

main().catch((err) => {
  console.error('[generate-recipe-assets] 失败:', err)
  process.exit(1)
})
