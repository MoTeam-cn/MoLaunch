/**
 * 版本设置页共享状态（模块级单例）
 *
 * 主文件 VersionSettings.vue 初始化后，
 * 子 Tab 组件直接复用同一份状态，避免 props 透传。
 */

import { ref, computed } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'
import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifinIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

const versionStore = useVersionStore()

const gameDir = ref('')
const effectiveDir = ref('')
const personalization = ref<tauri.VersionPersonalization | null>(null)

/** 当前选中的版本 ID */
const selectedId = computed(() => versionStore.selectedVersion)

/** 路径分隔符 */
const sep = computed(() => (gameDir.value.includes('\\') ? '\\' : '/'))

function joinPath(base: string, ...parts: string[]): string {
  return [base, ...parts].join(sep.value)
}

/** 版本文件夹路径 */
const versionFolder = computed(() => {
  if (!selectedId.value || !gameDir.value) return ''
  return joinPath(gameDir.value, 'versions', selectedId.value)
})

const savesFolder = computed(() => effectiveDir.value ? joinPath(effectiveDir.value, 'saves') : '')
const modsFolder = computed(() => effectiveDir.value ? joinPath(effectiveDir.value, 'mods') : '')
const resourcepacksFolder = computed(() => effectiveDir.value ? joinPath(effectiveDir.value, 'resourcepacks') : '')
const shaderpacksFolder = computed(() => effectiveDir.value ? joinPath(effectiveDir.value, 'shaderpacks') : '')

/** 图标选项 */
const iconOptions = [
  { value: '', label: '自动判断', icon: '' },
  { value: 'Grass', label: '草方块', icon: grassIcon },
  { value: 'CobbleStone', label: '圆石', icon: cobblestoneIcon },
  { value: 'CommandBlock', label: '命令方块', icon: commandBlockIcon },
  { value: 'GoldBlock', label: '金块', icon: goldBlockIcon },
  { value: 'Anvil', label: '铁砧', icon: anvilIcon },
  { value: 'Fabric', label: 'Fabric', icon: fabricIcon },
  { value: 'NeoForge', label: 'NeoForge', icon: neoforgeIcon },
  { value: 'RedstoneLampOn', label: '红石灯', icon: optifinIcon },
  { value: 'Egg', label: '蛋', icon: liteloaderIcon },
]

const displayTypeOptions = [
  { value: 0, label: '自动判断' },
  { value: 2, label: '可安装 Mod' },
  { value: 3, label: '原版类似' },
  { value: 5, label: '愚人节' },
  { value: 1, label: '隐藏' },
]

/** 推断版本类型 */
function inferVersionType(id: string): string {
  if (!id) return 'release'
  const lower = id.toLowerCase()
  if (lower.includes('neoforge')) return 'neoforge'
  if (lower.includes('forge')) return 'forge'
  if (lower.includes('fabric')) return 'fabric'
  if (lower.includes('optifine')) return 'optifine'
  if (lower.includes('liteloader')) return 'liteloader'
  if (/^\d{2}w\d{2}[a-z]/.test(id)) return 'snapshot'
  return 'release'
}

const typeMetaMap: Record<string, { icon: string; label: string }> = {
  release: { icon: grassIcon, label: '正式版' },
  snapshot: { icon: commandBlockIcon, label: '快照' },
  forge: { icon: anvilIcon, label: 'Forge' },
  neoforge: { icon: neoforgeIcon, label: 'NeoForge' },
  fabric: { icon: fabricIcon, label: 'Fabric' },
  optifine: { icon: optifinIcon, label: 'OptiFine' },
  liteloader: { icon: liteloaderIcon, label: 'LiteLoader' },
  old: { icon: cobblestoneIcon, label: '旧版' },
  fool: { icon: goldBlockIcon, label: '愚人节版' },
}

const currentMeta = computed(() => {
  if (!selectedId.value) return { icon: grassIcon, label: '其他' }
  return typeMetaMap[inferVersionType(selectedId.value)] ?? { icon: grassIcon, label: '其他' }
})

const currentLogo = computed(() => personalization.value?.logo ?? '')
const currentLogoIcon = computed(() => {
  const opt = iconOptions.find(o => o.value === currentLogo.value)
  return opt?.icon || currentMeta.value.icon
})

/** 是否支持 Mod */
const isModable = computed(() => {
  if (!selectedId.value) return false
  return ['forge', 'neoforge', 'fabric', 'optifine', 'liteloader'].includes(inferVersionType(selectedId.value))
})

/** 加载个性化数据 */
async function loadPersonalization() {
  if (!selectedId.value) {
    personalization.value = null
    return
  }
  try {
    personalization.value = await tauri.getVersionPersonalization(selectedId.value)
  } catch (e) {
    console.error('Failed to load personalization:', e)
  }
}

/** 初始化（主文件 onMounted 调用） */
async function initContext() {
  try {
    gameDir.value = await tauri.getGameDir()
    if (selectedId.value) {
      effectiveDir.value = await tauri.getVersionEffectiveDir(selectedId.value)
      await loadPersonalization()
    }
  } catch (e) {
    console.error('Failed to init version context:', e)
  }
}

/** 选中版本变化时刷新 effectiveDir */
async function refreshEffectiveDir() {
  if (selectedId.value) {
    effectiveDir.value = await tauri.getVersionEffectiveDir(selectedId.value)
  } else {
    effectiveDir.value = ''
  }
}

export function useVersionSettings() {
  return {
    selectedId,
    gameDir,
    effectiveDir,
    personalization,
    versionFolder,
    savesFolder,
    modsFolder,
    resourcepacksFolder,
    shaderpacksFolder,
    iconOptions,
    displayTypeOptions,
    currentMeta,
    currentLogo,
    currentLogoIcon,
    isModable,
    inferVersionType,
    loadPersonalization,
    initContext,
    refreshEffectiveDir,
  }
}
