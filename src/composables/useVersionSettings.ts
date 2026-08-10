/**
 * 版本设置页共享状态（模块级单例）
 *
 * 主文件 VersionSettings.vue 初始化后，
 * 子 Tab 组件直接复用同一份状态，避免 props 透传。
 */

import { ref, computed, watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import { inferVersionType, typeMetaMap } from '@/composables/useVersionMeta'
import { safeCall } from '@/utils/async'
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

/** 已安装版本类型映射（后端精确分析 JSON 的结果，供 inferVersionType 用作 backendType）
 *  形如 { "1.20.1-forge-47.4.10": "forge", "Zombie Invade 100 Days": "forge" }
 *  仅靠版本 ID 关键字匹配无法识别整合包版本（ID 不含 "forge" 字样），必须依赖后端类型 */
const installedVersionTypes = ref<Record<string, string>>({})

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

const currentMeta = computed(() => {
  if (!selectedId.value) return { icon: grassIcon, label: '其他' }
  const backendType = installedVersionTypes.value[selectedId.value]
  return typeMetaMap[inferVersionType(selectedId.value, undefined, backendType)] ?? { icon: grassIcon, label: '其他' }
})

const currentLogo = computed(() => personalization.value?.logo ?? '')
const currentLogoIcon = computed(() => {
  const opt = iconOptions.find(o => o.value === currentLogo.value)
  return opt?.icon || currentMeta.value.icon
})

/**
 * 根据自定义 logo + 版本 ID 解析图标（供版本列表/选择页使用）
 *
 * 与 useVersionMeta.ts 的 resolveVersionIcon(type) 区别：
 *   - useVersionMeta.resolveVersionIcon(type)：仅按版本类型查表取图标（type→icon）
 *   - 本函数 resolveVersionIconWithLogo(logo, versionId, explicitType?)：含 logo 优先策略，
 *     logo 非空时优先使用 iconOptions 中匹配的图标，否则按 versionId 推断类型后查表
 *
 * 优先用显式传入的 explicitType（调用方已知后端类型时直接传，避免依赖 installedVersionTypes 的加载时机）
 * 其次用 installedVersionTypes（模块级缓存）
 */
function resolveVersionIconWithLogo(logo: string, versionId: string, explicitType?: string): string {
  if (logo) {
    const opt = iconOptions.find(o => o.value === logo)
    if (opt?.icon) return opt.icon
  }
  const backendType = explicitType ?? installedVersionTypes.value[versionId]
  const inferred = inferVersionType(versionId, undefined, backendType)
  return typeMetaMap[inferred]?.icon ?? grassIcon
}

/** 是否支持 Mod */
const isModable = computed(() => {
  if (!selectedId.value) return false
  const backendType = installedVersionTypes.value[selectedId.value]
  return ['forge', 'neoforge', 'fabric', 'optifine', 'liteloader'].includes(inferVersionType(selectedId.value, undefined, backendType))
})

/** 当前版本是否支持光影（后端检查 OptiFine/Iris，与光影管理页 PackTab 逻辑一致） */
const shaderAvailable = ref(true)

/** 检查当前版本的光影可用性（is_packs_available 对 Shader 检查 OptiFine/Iris） */
async function checkShaderAvailable() {
  if (!selectedId.value) {
    shaderAvailable.value = false
    return
  }
  try {
    shaderAvailable.value = await tauri.isPacksAvailable(selectedId.value, 'shader')
  } catch {
    shaderAvailable.value = true
  }
}

// 切换版本时重新检查光影可用性
watch(selectedId, (val) => {
  if (val) checkShaderAvailable()
  else shaderAvailable.value = false
})

/** 加载个性化数据 */
async function loadPersonalization() {
  if (!selectedId.value) {
    personalization.value = null
    return
  }
  const data = await safeCall(() => tauri.getVersionPersonalization(selectedId.value!), 'load personalization')
  if (data) personalization.value = data
}

/** 初始化（主文件 onMounted 调用） */
async function initContext() {
  await safeCall(async () => {
    gameDir.value = await tauri.getGameDir()
    // 加载已安装版本类型映射（用于 inferVersionType 的 backendType，正确识别整合包版本类型）
    await safeCall(async () => {
      const vwt = await tauri.listInstalledVersionsWithType()
      const typeMap: Record<string, string> = {}
      vwt.forEach(v => { typeMap[v.id] = v.version_type })
      installedVersionTypes.value = typeMap
    }, 'load installed version types')
    if (selectedId.value) {
      effectiveDir.value = await tauri.getVersionEffectiveDir(selectedId.value)
      await loadPersonalization()
      checkShaderAvailable()
    }
  }, 'init version context')
}

/** 选中版本变化时刷新 effectiveDir */
async function refreshEffectiveDir() {
  if (selectedId.value) {
    effectiveDir.value = await tauri.getVersionEffectiveDir(selectedId.value)
  } else {
    effectiveDir.value = ''
  }
}

/** 刷新已安装版本类型映射（版本列表更新、安装/卸载后调用）
 *
 * @param vwtList 可选，已获取的已安装版本列表，避免重复调用 IPC
 */
async function refreshInstalledVersionTypes(vwtList?: { id: string; version_type: string }[]) {
  await safeCall(async () => {
    const vwt = vwtList ?? await tauri.listInstalledVersionsWithType()
    const typeMap: Record<string, string> = {}
    vwt.forEach(v => { typeMap[v.id] = v.version_type })
    installedVersionTypes.value = typeMap
  }, 'refresh installed version types')
}

export function useVersionSettings() {
  return {
    selectedId,
    gameDir,
    effectiveDir,
    personalization,
    installedVersionTypes,
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
    shaderAvailable,
    inferVersionType,
    resolveVersionIconWithLogo,
    loadPersonalization,
    initContext,
    refreshEffectiveDir,
    refreshInstalledVersionTypes,
  }
}
