/**
 * 种子地图常量与样式定义（OpenLayers 版）
 *
 * 包含：
 * - 群系 ID → RGB 调色板（与 cubiomes util.c initBiomeColors 一致）
 * - 结构类型 → 图标元数据（形状 + 颜色 + 中文标签）
 * - OL Style 工厂：根据结构类型生成 OpenLayers 样式（不同形状区分类型）
 *
 * 从 SeedMap.vue 拆出，避免 Vue 组件超 300 行。
 * 不再包含 canvas draw 函数——OL 内置渲染，无需手写。
 */
import Style from 'ol/style/Style'
import Circle from 'ol/style/Circle'
import Fill from 'ol/style/Fill'
import Stroke from 'ol/style/Stroke'
import Icon from 'ol/style/Icon'

// ===== 群系调色板 =====
export const BIOME_COLORS: Record<number, [number, number, number]> = {
  0: [0, 0, 255], 1: [145, 199, 135], 2: [217, 200, 138], 3: [96, 96, 145],
  4: [86, 168, 81], 5: [74, 124, 103], 6: [107, 138, 124], 7: [63, 117, 200],
  8: [255, 0, 0], 9: [128, 128, 255], 10: [112, 178, 219], 11: [160, 224, 255],
  12: [255, 255, 255], 13: [160, 167, 167], 14: [255, 0, 255], 15: [160, 0, 160],
  16: [250, 222, 145], 17: [217, 200, 138], 18: [86, 168, 81], 19: [74, 124, 103],
  20: [119, 119, 119], 21: [104, 168, 81], 22: [122, 197, 122], 23: [91, 158, 71],
  24: [0, 0, 100], 25: [162, 162, 132], 26: [255, 255, 255], 27: [104, 168, 81],
  28: [122, 197, 122], 29: [64, 109, 47], 30: [74, 124, 103], 31: [74, 124, 103],
  32: [103, 158, 122], 33: [103, 158, 122], 34: [96, 96, 145], 35: [180, 219, 132],
  36: [199, 224, 145], 37: [167, 75, 31], 38: [217, 130, 64], 39: [184, 80, 31],
  40: [128, 128, 255], 41: [128, 128, 255], 42: [128, 128, 255], 43: [128, 128, 255],
  44: [0, 0, 200], 45: [0, 0, 165], 46: [0, 0, 200], 47: [0, 0, 144],
  48: [0, 0, 144], 49: [0, 0, 144], 50: [0, 0, 144],
  170: [60, 31, 0], 171: [180, 50, 50], 172: [50, 180, 180], 173: [40, 40, 40],
  174: [110, 80, 50], 175: [70, 130, 90],
  177: [180, 220, 180], 178: [100, 130, 110], 179: [200, 220, 230],
  180: [180, 180, 200], 181: [220, 230, 240], 182: [200, 200, 180],
  183: [40, 40, 60], 184: [70, 100, 60], 185: [255, 180, 220], 186: [180, 170, 200],
  129: [180, 220, 180], 130: [217, 200, 138], 131: [96, 96, 145],
  132: [104, 168, 81], 133: [74, 124, 103], 134: [107, 138, 124],
  140: [200, 220, 230], 149: [104, 168, 81], 150: [91, 158, 71],
  155: [104, 168, 81], 156: [122, 197, 122], 157: [64, 109, 47],
  158: [74, 124, 103], 160: [103, 158, 122], 161: [103, 158, 122],
  162: [96, 96, 145], 163: [180, 219, 132], 164: [199, 224, 145],
  165: [167, 75, 31], 168: [104, 168, 81], 169: [122, 197, 122],
}

export const DEFAULT_COLOR: [number, number, number] = [200, 200, 200]

export function biomeColor(id: number): [number, number, number] {
  return BIOME_COLORS[id] ?? DEFAULT_COLOR
}

// ===== 结构图标定义 =====
export type StructureShape = 'circle' | 'square' | 'triangle' | 'diamond'

export interface StructureIconDef {
  shape: StructureShape
  color: string
  label: string
}

/**
 * 结构类型 → 图标定义。形状语义：方形=建筑，三角=神殿/前哨，圆=海洋/水，菱=宝藏/传送门
 *
 * 每个结构的 color 同时作为：
 * - 无 webp 图标时 OL Shape 渲染的填充色
 * - 高亮时的描边色
 */
export const STRUCTURE_ICONS: Record<string, StructureIconDef> = {
  Village:         { shape: 'square',   color: '#8B4513', label: '村庄' },
  Desert_Pyramid:  { shape: 'triangle', color: '#FFD700', label: '沙漠神殿' },
  Jungle_Temple:   { shape: 'triangle', color: '#228B22', label: '丛林神庙' },
  Swamp_Hut:       { shape: 'square',   color: '#800080', label: '沼泽小屋' },
  Igloo:           { shape: 'square',   color: '#ADD8E6', label: '雪屋' },
  Ocean_Ruin:      { shape: 'circle',   color: '#5F9EA0', label: '海底废墟' },
  Shipwreck:       { shape: 'diamond',  color: '#8B4513', label: '沉船' },
  Monument:        { shape: 'circle',   color: '#00CED1', label: '海底神殿' },
  Mansion:         { shape: 'square',   color: '#5C3317', label: '林地府邸' },
  Outpost:         { shape: 'triangle', color: '#A9A9A9', label: '掠夺者前哨站' },
  Ruined_Portal:   { shape: 'diamond',  color: '#9370DB', label: '废弃传送门' },
  Ruined_Portal_N: { shape: 'diamond',  color: '#9370DB', label: '废弃传送门（下界）' },
  Ancient_City:    { shape: 'square',   color: '#2F4F4F', label: '远古城市' },
  Treasure:        { shape: 'diamond',  color: '#FFA500', label: '埋藏宝藏' },
  Desert_Well:     { shape: 'circle',   color: '#FFE4B5', label: '沙漠水井' },
  Geode:           { shape: 'diamond',  color: '#BA55D3', label: '紫水晶洞' },
  Trail_Ruins:     { shape: 'square',   color: '#CD853F', label: '遗迹废墟' },
  Trial_Chambers:  { shape: 'square',   color: '#DAA520', label: '试炼密室' },
  Mineshaft:       { shape: 'square',   color: '#6B5B3A', label: '废弃矿井' },
  Fortress:        { shape: 'square',   color: '#FF4500', label: '下界要塞' },
  Bastion:         { shape: 'square',   color: '#FF1493', label: '堡垒遗迹' },
  End_City:        { shape: 'square',   color: '#FFB6C1', label: '末地城' },
  End_Gateway:     { shape: 'circle',   color: '#E6E6FA', label: '末地折跃门' },
  Slime_Chunks:    { shape: 'circle',   color: '#44FF44', label: '史莱姆区块' },
  // 扩展结构（方案 A）—— 颜色取自 docs/Map/prompt-structures.md
  Ravine:                  { shape: 'circle',  color: '#7A6B5A', label: '峡谷' },
  Mega_Ravine:             { shape: 'circle',  color: '#5A4B3A', label: '巨型峡谷' },
  Underwater_Ravine:       { shape: 'circle',  color: '#4A7A8B', label: '水下峡谷' },
  Mega_Underwater_Ravine:  { shape: 'circle',  color: '#3A6A7B', label: '巨型水下峡谷' },
  Nether_Fossil:           { shape: 'diamond', color: '#D4C4A8', label: '下界化石' },
  Fossil:                  { shape: 'diamond', color: '#E8D5B0', label: '化石' },
  Fossil_Diamond:          { shape: 'diamond', color: '#5DCED1', label: '钻石化石' },
}

export function getStructIcon(stype: string): StructureIconDef {
  return STRUCTURE_ICONS[stype] ?? { shape: 'circle', color: '#FFF', label: stype }
}

// ===== 结构图标 URL =====
// Vite 5 推荐语法：query:'?url' + import:'default'，直接返回 url 字符串。
const iconUrlMap: Record<string, string> = {}
const globModules = import.meta.glob('@/assets/structures/*.webp', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>
for (const [key, val] of Object.entries(globModules)) {
  const m = key.match(/\/([^/]+)\.webp$/)
  if (m && typeof val === 'string' && val) iconUrlMap[m[1]] = val
}

console.log('[seedmap] iconUrlMap loaded:', Object.keys(iconUrlMap).length, 'icons, sample:', iconUrlMap['village'] ?? '(empty)')

/**
 * 结构名 → 图标文件名别名映射
 *
 * 部分结构变体共用同一图标（参考 docs/Map/prompt-structures.md §图标资源 URL）：
 * - Mega_Ravine / Underwater_Ravine / Mega_Underwater_Ravine → ravine.webp
 *   （canyon carver 系列视觉相似，原站统一用 ravine.webp）
 */
const ICON_NAME_ALIASES: Record<string, string> = {
  mega_ravine: 'ravine',
  underwater_ravine: 'ravine',
  mega_underwater_ravine: 'ravine',
}

/** 获取结构图标 URL（供 <img> 和 OL Icon 使用） */
export function getStructIconUrl(stype: string): string {
  const raw = stype.toLowerCase()
  const name = ICON_NAME_ALIASES[raw] ?? raw
  return iconUrlMap[name] ?? ''
}

// ===== OL Style 工厂（缓存样式避免重复创建） =====
const styleCache = new Map<string, Style>()

/**
 * 根据结构类型生成 OL Style
 *
 * 优先使用 webp 图标（src/assets/structures/*.webp）；
 * 若该结构无 webp 资源（如 Mineshaft），则用 OL Circle + STRUCTURE_ICONS.color
 * 渲染几何形状作为 fallback，确保所有结构都有可见标记。
 */
export function getStructStyle(stype: string, highlighted = false): Style {
  const key = stype + (highlighted ? '_h' : '')
  const cached = styleCache.get(key)
  if (cached) return cached
  const url = getStructIconUrl(stype)
  const def = getStructIcon(stype)
  let style: Style
  if (url) {
    // 有 webp 图标：scale=0.6 → 约 19px（图标偏小在高密度结构区不易辨识）
    // 不设 crossOrigin：Tauri webview 自定义协议（tauri://localhost）不支持 CORS 预检，
    // 设 crossOrigin:'anonymous' 会导致图片加载静默失败，OL 退化为不可见（用户观感"红点"）
    const scale = highlighted ? 0.8 : 0.6
    style = new Style({
      image: new Icon({
        src: url,
        scale,
        anchor: [0.5, 0.5],
        anchorXUnits: 'fraction',
        anchorYUnits: 'fraction',
      }),
    })
  } else {
    // 无 webp 图标：用 OL Circle 渲染几何形状 fallback
    // highlighted 时半径加大 + 白色描边，便于识别 hover/click
    const radius = highlighted ? 8 : 6
    style = new Style({
      image: new Circle({
        radius,
        fill: new Fill({ color: def.color }),
        stroke: new Stroke({
          color: highlighted ? '#FFFFFF' : 'rgba(0,0,0,0.6)',
          width: highlighted ? 2 : 1,
        }),
      }),
    })
  }
  styleCache.set(key, style)
  return style
}

/** 出生点/要塞标记样式（十字圆点） */
export function getMarkerStyle(color: string): Style {
  const cached = styleCache.get('marker_' + color)
  if (cached) return cached
  const style = new Style({
    image: new Circle({
      radius: 6,
      fill: new Fill({ color }),
      stroke: new Stroke({ color: '#FFF', width: 2 }),
    }),
  })
  styleCache.set('marker_' + color, style)
  return style
}

/** 点击坐标标记样式（黄色圆圈） */
export function getClickMarkerStyle(): Style {
  const cached = styleCache.get('click_marker')
  if (cached) return cached
  const style = new Style({
    image: new Circle({
      radius: 8,
      fill: new Fill({ color: 'rgba(255,255,0,0.4)' }),
      stroke: new Stroke({ color: '#FF0', width: 2 }),
    }),
  })
  styleCache.set('click_marker', style)
  return style
}
