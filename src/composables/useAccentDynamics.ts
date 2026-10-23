/**
 * 主色阶动态渲染
 *
 * 沿色相路径对 50~950 档主色阶做连续插值并写入 CSS 变量，得到持续流动的主色；
 * 档位表与展示名在运行期解析，未选定档位时完全走静态主色，不产生额外开销。
 */

import { computed, ref } from 'vue'
import { applyPrimaryColor, hslToHex } from '@/utils/color'
import { useSettingsStore } from '@/stores/settings'

/** 档位记录：[编号, 展示名密文, 色相路径(度), 饱和度, 亮度, 单圈时长(ms)] */
type ProfileRow = [string, string, number[], number, number, number]

const PROFILE_ROWS: ProfileRow[] = [
  ['p1', 'i/Ksg+nmnvD+ytLH', [188, 224, 268], 0.86, 0.55, 9200],
  ['p2', 'iuq5j+v+n9jIxPry', [16, 336, 268], 0.88, 0.56, 12400],
  ['p3', 'i9ucgNnYkdDCy9f8', [204, 176, 158], 0.72, 0.52, 14600],
  ['p4', 'i8ScjubekffCxe/e', [324, 286, 258], 0.74, 0.63, 11200],
  ['p5', 'iNWWg9HGkc7dxPPw', [4, 118, 232], 0.8, 0.58, 18000],
]

const STORE_KEY = 'molaunch.ui.accentProfile'
const TICK_MS = 80

interface AccentProfile {
  id: string
  label: string
  hues: number[]
  sat: number
  light: number
  period: number
}

const PAD = 'ml-flow'

/** 密文还原（按 PAD 逐字节异或后按 UTF-8 解码） */
function reveal(raw: string): string {
  try {
    const bin = atob(raw)
    const bytes = new Uint8Array(bin.length)
    for (let i = 0; i < bin.length; i += 1) {
      bytes[i] = bin.charCodeAt(i) ^ PAD.charCodeAt(i % PAD.length)
    }
    return new TextDecoder().decode(bytes)
  } catch {
    return ''
  }
}

const PROFILES: AccentProfile[] = PROFILE_ROWS.map(([id, label, hues, sat, light, period]) => ({
  id,
  label: reveal(label),
  hues,
  sat,
  light,
  period,
}))

const available = ref(false)
const activeId = ref('')
let timer: ReturnType<typeof setInterval> | null = null
let epoch = 0

function readStore(): { version: number; id: string } {
  try {
    const raw = localStorage.getItem(STORE_KEY)
    if (!raw) return { version: 0, id: '' }
    const [version, id] = raw.split('|')
    return { version: Number(version) || 0, id: id ?? '' }
  } catch {
    return { version: 0, id: '' }
  }
}

function writeStore(version: number, id: string): void {
  try {
    localStorage.setItem(STORE_KEY, version + '|' + id)
  } catch {
    console.warn('[Accent] 档位本地写入失败')
  }
}

/** 按进度在色相路径上取色（跨 0/360 走最短弧） */
function hueAt(stops: number[], progress: number): number {
  if (stops.length < 2) return stops[0] ?? 0
  const span = progress * stops.length
  const index = Math.floor(span) % stops.length
  const local = span - Math.floor(span)
  const from = stops[index]
  const to = stops[(index + 1) % stops.length]
  let delta = to - from
  if (delta > 180) delta -= 360
  if (delta < -180) delta += 360
  return (from + delta * local + 360) % 360
}

function paint(profile: AccentProfile, elapsed: number): void {
  const progress = (elapsed % profile.period) / profile.period
  applyPrimaryColor(hslToHex(hueAt(profile.hues, progress) / 360, profile.sat, profile.light))
}

function stop(): void {
  if (timer !== null) {
    clearInterval(timer)
    timer = null
  }
  epoch = 0
  activeId.value = ''
  applyPrimaryColor(useSettingsStore().primaryColor)
}

function start(profile: AccentProfile): void {
  if (timer !== null) clearInterval(timer)
  activeId.value = profile.id
  epoch = Date.now()
  paint(profile, 0)
  timer = setInterval(() => paint(profile, Date.now() - epoch), TICK_MS)
}

/** 切换档位（空值 = 回到静态主色） */
function selectProfile(id: string): void {
  const profile = PROFILES.find((item) => item.id === id)
  if (!profile) {
    stop()
    writeStore(available.value ? 1 : 0, '')
    return
  }
  if (!available.value) return
  start(profile)
  writeStore(1, profile.id)
}

/** 标记档位表可用（幂等） */
function enable(): void {
  if (available.value) return
  available.value = true
  writeStore(1, activeId.value)
}

/** 启动时按本地记录恢复 */
function restore(): void {
  const saved = readStore()
  if (saved.version < 1) return
  available.value = true
  const profile = PROFILES.find((item) => item.id === saved.id)
  if (profile) start(profile)
}

const options = computed(() => [
  { label: '跟随主题色', value: '' },
  ...PROFILES.map((item) => ({ label: item.label, value: item.id })),
])

export function useAccentDynamics() {
  return { available, activeId, options, selectProfile, enable, restore }
}
