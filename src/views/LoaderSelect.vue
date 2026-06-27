<script setup lang="ts">
/**
 * 加载器选择组件
 * 选择后自动收起，标题显示已选内容，支持清除
 */

import { ref, watch, onMounted, computed } from 'vue'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import { ChevronLeftIcon, XMarkIcon } from '@heroicons/vue/24/outline'

import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

interface Props {
  mcVersion: string
}

const props = defineProps<Props>()
const emit = defineEmits<{ back: []; installing: [] }>()

const forgeVersions = ref<string[]>([])
const neoforgeVersions = ref<{ version: string; recommended: boolean }[]>([])
const fabricVersions = ref<{ version: string; stable: boolean }[]>([])
const optifineVersions = ref<{ display_name: string; is_preview: boolean }[]>([])
const liteloaderVersions = ref<string[]>([])

const selectedForge = ref<string | null>(null)
const selectedNeoforge = ref<string | null>(null)
const selectedFabric = ref<string | null>(null)
const selectedOptifine = ref<string | null>(null)
const selectedLiteloader = ref<string | null>(null)

const loading = ref(true)
const expandedCards = ref<Set<string>>(new Set())
const installing = ref(false)
const compatible = ref(true)
const conflictMsg = ref('')

const mcNum = computed(() => {
  const parts = props.mcVersion.split('.')
  return (parseInt(parts[0]) || 0) * 10000 + (parseInt(parts[1]) || 0) * 100 + (parseInt(parts[2]) || 0)
})

const showForge = computed(() => mcNum.value >= 10501)
const showNeoforge = computed(() => mcNum.value >= 12001)
const showFabric = computed(() => mcNum.value > 11300)
const showLiteloader = computed(() => mcNum.value <= 11202) // <= 1.12.2

// 过滤 OptiFine：只显示与当前 MC 版本匹配的
const filteredOptifine = computed(() => {
  return optifineVersions.value.filter(v => {
    // display_name 格式: "1.13.2 HD U E7"，前面是 MC 版本号
    const match = v.display_name.match(/^([\d.]+)\s/)
    if (!match) return false
    return match[1] === props.mcVersion
  })
})

onMounted(async () => {
  loading.value = true
  const tasks: Promise<void>[] = []
  if (showForge.value) {
    tasks.push(tauri.listForgeVersions(props.mcVersion).then(v => { forgeVersions.value = v }).catch(() => {}))
  }
  if (showNeoforge.value) {
    tasks.push(tauri.listNeoforgeVersions(props.mcVersion).then(v => {
      // 倒序：最新版在前
      neoforgeVersions.value = v.reverse()
    }).catch(() => {}))
  }
  if (showFabric.value) {
    tasks.push(tauri.listFabricVersions().then(v => { fabricVersions.value = v }).catch(() => {}))
  }
  if (showLiteloader.value) {
    tasks.push(tauri.listLiteloaderVersions(props.mcVersion).then(v => { liteloaderVersions.value = v }).catch(() => {}))
  }
  tasks.push(tauri.listOptifineVersions().then(v => { optifineVersions.value = v }).catch(() => {}))
  await Promise.all(tasks)
  loading.value = false
})

async function checkCompatibility() {
  try {
    const ok = await tauri.validateLoaders(
      props.mcVersion,
      selectedForge.value || undefined,
      selectedNeoforge.value || undefined,
      selectedFabric.value || undefined,
      selectedOptifine.value || undefined,
    )
    compatible.value = ok
    conflictMsg.value = ok ? '' : '所选加载器组合不兼容'
  } catch {
    compatible.value = true
    conflictMsg.value = ''
  }
}

watch([selectedForge, selectedNeoforge, selectedFabric, selectedOptifine, selectedLiteloader], checkCompatibility)

// 前端兼容性检查：返回不兼容原因，null 表示兼容
function getLoaderError(loader: string): string | null {
  const forge = selectedForge.value
  const neoforge = selectedNeoforge.value
  const fabric = selectedFabric.value
  const optifine = selectedOptifine.value

  if (loader === 'forge') {
    if (fabric) return '与 Fabric 不兼容'
    if (neoforge) return '与 NeoForge 不兼容'
  }
  if (loader === 'neoforge') {
    if (forge) return '与 Forge 不兼容'
    if (fabric) return '与 Fabric 不兼容'
    if (optifine) return '与 OptiFine 不兼容'
  }
  if (loader === 'fabric') {
    if (forge) return '与 Forge 不兼容'
    if (neoforge) return '与 NeoForge 不兼容'
  }
  if (loader === 'optifine') {
    if (neoforge) return '与 NeoForge 不兼容'
  }
  // LiteLoader 与所有其他加载器兼容
  return null
}

// 加载器是否被禁用（其他已选加载器导致不兼容）
function isLoaderDisabled(loader: string): boolean {
  return getLoaderError(loader) !== null && !isLoaderSelected(loader)
}

function isLoaderSelected(loader: string): boolean {
  if (loader === 'forge') return !!selectedForge.value
  if (loader === 'neoforge') return !!selectedNeoforge.value
  if (loader === 'fabric') return !!selectedFabric.value
  if (loader === 'optifine') return !!selectedOptifine.value
  if (loader === 'liteloader') return !!selectedLiteloader.value
  return false
}

function toggleCard(id: string) {
  if (expandedCards.value.has(id)) {
    expandedCards.value.delete(id)
  } else {
    expandedCards.value.add(id)
  }
}

function isExpanded(id: string): boolean {
  return expandedCards.value.has(id)
}

function selectAndCollapse(type: string, value: string | null, current: string | null) {
  const newVal = current === value ? null : value
  if (type === 'forge') selectedForge.value = newVal
  else if (type === 'neoforge') selectedNeoforge.value = newVal
  else if (type === 'fabric') selectedFabric.value = newVal
  else if (type === 'optifine') selectedOptifine.value = newVal
  else if (type === 'liteloader') selectedLiteloader.value = newVal
  expandedCards.value.delete(type)
}

function clearSelection(type: string, e: Event) {
  e.stopPropagation()
  if (type === 'forge') selectedForge.value = null
  else if (type === 'neoforge') selectedNeoforge.value = null
  else if (type === 'fabric') selectedFabric.value = null
  else if (type === 'optifine') selectedOptifine.value = null
  else if (type === 'liteloader') selectedLiteloader.value = null
}

function getInstanceName(): string {
  let name = props.mcVersion
  if (selectedFabric.value) name += `-Fabric${selectedFabric.value}`
  if (selectedForge.value) name += `-Forge_${selectedForge.value}`
  if (selectedNeoforge.value) name += `-NeoForge_${selectedNeoforge.value}`
  if (selectedOptifine.value) name += `-OptiFine`
  if (selectedLiteloader.value) name += `-LiteLoader_${selectedLiteloader.value}`
  return name
}

async function handleInstall() {
  installing.value = true
  emit('installing')
  try {
    await tauri.installMerged(
      props.mcVersion,
      selectedForge.value || undefined,
      selectedNeoforge.value || undefined,
      selectedFabric.value || undefined,
      selectedOptifine.value || undefined,
      selectedLiteloader.value || undefined,
      getInstanceName(),
    )
    showSuccess(`${getInstanceName()} 安装完成`)
  } catch (e) {
    showError('安装失败', String(e))
  } finally {
    installing.value = false
  }
}

const hasSelection = computed(() =>
  selectedForge.value || selectedNeoforge.value || selectedFabric.value || selectedOptifine.value || selectedLiteloader.value
)
</script>

<template>
  <div class="flex flex-col h-full animate-slide-in">
    <!-- 顶栏 -->
    <div class="px-6 py-4 bg-white border-b border-gray-300 flex items-center gap-3 shrink-0">
      <button class="p-1.5 hover:bg-gray-100 rounded-lg transition-colors" @click="emit('back')">
        <ChevronLeftIcon class="w-5 h-5 text-gray-600" />
      </button>
      <div>
        <h2 class="text-lg font-semibold text-gray-900">选择加载器</h2>
        <p class="text-xs text-gray-500">Minecraft {{ mcVersion }} — 可选安装 Mod 加载器</p>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="animate-spin rounded-full h-10 w-10 border-2 border-gray-200 border-t-primary-600 mx-auto"></div>
        <p class="text-sm text-gray-500 mt-4">不要急哦，正在加急请求各项资源...</p>
      </div>
    </div>

    <!-- 加载器列表 -->
    <div v-else class="flex-1 overflow-y-auto p-6 space-y-3">
      <!-- 兼容性警告 -->
      <div v-if="!compatible" class="p-3 bg-red-50 border border-red-300 rounded-lg text-sm text-red-700">
        {{ conflictMsg }}
      </div>

      <!-- Forge -->
      <div
        v-if="showForge && forgeVersions.length > 0"
        class="bg-white rounded-lg border overflow-hidden transition-colors"
        :class="isLoaderDisabled('forge') ? 'border-gray-200 opacity-60' : 'border-gray-300'"
      >
        <div
          class="flex items-center justify-between px-4 py-3 transition-colors"
          :class="isLoaderDisabled('forge') ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
          @click="!isLoaderDisabled('forge') && toggleCard('forge')"
        >
          <div class="flex items-center gap-2 min-w-0">
            <img :src="anvilIcon" class="w-5 h-5 rounded shrink-0" />
            <span class="text-sm font-medium text-gray-900 shrink-0">Forge</span>
            <span v-if="selectedForge" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-orange-100 text-orange-700 font-medium">
              {{ selectedForge }}
              <button class="p-0.5 -mr-1 hover:bg-orange-200 rounded-full transition-colors" @click="clearSelection('forge', $event)">
                <XMarkIcon class="w-3 h-3" />
              </button>
            </span>
            <span v-if="isLoaderDisabled('forge')" class="text-xs text-red-500 ml-1">{{ getLoaderError('forge') }}</span>
          </div>
          <svg class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0" :class="{ 'rotate-180': isExpanded('forge') }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="grid transition-all duration-300 ease-in-out" :style="{ gridTemplateRows: isExpanded('forge') ? '1fr' : '0fr' }">
          <div class="overflow-hidden min-h-0">
            <div class="border-t border-gray-200 p-3 space-y-1.5 max-h-48 overflow-y-auto">
              <div
                v-for="(ver, idx) in forgeVersions"
                :key="ver"
                class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
                :class="selectedForge === ver
                  ? 'border-orange-400 bg-orange-50 shadow-sm'
                  : 'border-gray-200 hover:border-orange-300 hover:bg-orange-50/50'"
                @click="selectAndCollapse('forge', ver, selectedForge)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm" :class="selectedForge === ver ? 'text-orange-800 font-medium' : 'text-gray-700'">{{ ver }}</span>
                  <span v-if="idx === 0" class="text-xs px-1.5 py-0.5 rounded bg-orange-100 text-orange-700 font-medium">最新版</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- NeoForge -->
      <div
        v-if="showNeoforge && neoforgeVersions.length > 0"
        class="bg-white rounded-lg border overflow-hidden transition-colors"
        :class="isLoaderDisabled('neoforge') ? 'border-gray-200 opacity-60' : 'border-gray-300'"
      >
        <div
          class="flex items-center justify-between px-4 py-3 transition-colors"
          :class="isLoaderDisabled('neoforge') ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
          @click="!isLoaderDisabled('neoforge') && toggleCard('neoforge')"
        >
          <div class="flex items-center gap-2 min-w-0">
            <img :src="neoforgeIcon" class="w-5 h-5 rounded shrink-0" />
            <span class="text-sm font-medium text-gray-900 shrink-0">NeoForge</span>
            <span v-if="selectedNeoforge" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-purple-100 text-purple-700 font-medium">
              {{ mcVersion }}.{{ selectedNeoforge }}
              <button class="p-0.5 -mr-1 hover:bg-purple-200 rounded-full transition-colors" @click="clearSelection('neoforge', $event)">
                <XMarkIcon class="w-3 h-3" />
              </button>
            </span>
            <span v-if="isLoaderDisabled('neoforge')" class="text-xs text-red-500 ml-1">{{ getLoaderError('neoforge') }}</span>
          </div>
          <svg class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0" :class="{ 'rotate-180': isExpanded('neoforge') }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="grid transition-all duration-300 ease-in-out" :style="{ gridTemplateRows: isExpanded('neoforge') ? '1fr' : '0fr' }">
          <div class="overflow-hidden min-h-0">
            <div class="border-t border-gray-200 p-3 space-y-1.5 max-h-48 overflow-y-auto">
              <div
                v-for="ver in neoforgeVersions"
                :key="ver.version"
                class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
                :class="selectedNeoforge === ver.version
                  ? 'border-purple-400 bg-purple-50 shadow-sm'
                  : 'border-gray-200 hover:border-purple-300 hover:bg-purple-50/50'"
                @click="selectAndCollapse('neoforge', ver.version, selectedNeoforge)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm" :class="selectedNeoforge === ver.version ? 'text-purple-800 font-medium' : 'text-gray-700'">{{ mcVersion }}.{{ ver.version }}</span>
                  <span v-if="ver.recommended" class="text-xs px-1.5 py-0.5 rounded bg-green-100 text-green-700 font-medium">推荐</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Fabric -->
      <div
        v-if="showFabric && fabricVersions.length > 0"
        class="bg-white rounded-lg border overflow-hidden transition-colors"
        :class="isLoaderDisabled('fabric') ? 'border-gray-200 opacity-60' : 'border-gray-300'"
      >
        <div
          class="flex items-center justify-between px-4 py-3 transition-colors"
          :class="isLoaderDisabled('fabric') ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
          @click="!isLoaderDisabled('fabric') && toggleCard('fabric')"
        >
          <div class="flex items-center gap-2 min-w-0">
            <img :src="fabricIcon" class="w-5 h-5 rounded shrink-0" />
            <span class="text-sm font-medium text-gray-900 shrink-0">Fabric</span>
            <span v-if="selectedFabric" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-700 font-medium">
              {{ selectedFabric }}
              <button class="p-0.5 -mr-1 hover:bg-blue-200 rounded-full transition-colors" @click="clearSelection('fabric', $event)">
                <XMarkIcon class="w-3 h-3" />
              </button>
            </span>
            <span v-if="isLoaderDisabled('fabric')" class="text-xs text-red-500 ml-1">{{ getLoaderError('fabric') }}</span>
          </div>
          <svg class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0" :class="{ 'rotate-180': isExpanded('fabric') }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="grid transition-all duration-300 ease-in-out" :style="{ gridTemplateRows: isExpanded('fabric') ? '1fr' : '0fr' }">
          <div class="overflow-hidden min-h-0">
            <div class="border-t border-gray-200 p-3 space-y-1.5 max-h-48 overflow-y-auto">
              <div
                v-for="ver in fabricVersions"
                :key="ver.version"
                class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
                :class="selectedFabric === ver.version
                  ? 'border-blue-400 bg-blue-50 shadow-sm'
                  : 'border-gray-200 hover:border-blue-300 hover:bg-blue-50/50'"
                @click="selectAndCollapse('fabric', ver.version, selectedFabric)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm" :class="selectedFabric === ver.version ? 'text-blue-800 font-medium' : 'text-gray-700'">{{ ver.version }}</span>
                  <span v-if="ver.stable" class="text-xs px-1.5 py-0.5 rounded bg-green-100 text-green-700 font-medium">稳定版</span>
                  <span v-else class="text-xs px-1.5 py-0.5 rounded bg-yellow-100 text-yellow-700 font-medium">测试版</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- OptiFine -->
      <div
        class="bg-white rounded-lg border overflow-hidden transition-colors"
        :class="isLoaderDisabled('optifine') ? 'border-gray-200 opacity-60' : 'border-gray-300'"
      >
        <div
          class="flex items-center justify-between px-4 py-3 transition-colors"
          :class="isLoaderDisabled('optifine') || filteredOptifine.length === 0 ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
          @click="filteredOptifine.length > 0 && !isLoaderDisabled('optifine') && toggleCard('optifine')"
        >
          <div class="flex items-center gap-2 min-w-0">
            <img :src="optifineIcon" class="w-5 h-5 rounded shrink-0" />
            <span class="text-sm font-medium text-gray-900 shrink-0">OptiFine</span>
            <span v-if="selectedOptifine" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700 font-medium max-w-[200px] truncate">
              {{ selectedOptifine }}
              <button class="p-0.5 -mr-1 hover:bg-green-200 rounded-full transition-colors shrink-0" @click="clearSelection('optifine', $event)">
                <XMarkIcon class="w-3 h-3" />
              </button>
            </span>
            <span v-if="isLoaderDisabled('optifine')" class="text-xs text-red-500 ml-1">{{ getLoaderError('optifine') }}</span>
          </div>
          <svg v-if="filteredOptifine.length > 0" class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0" :class="{ 'rotate-180': isExpanded('optifine') }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </div>
        <!-- 有版本时展开列表 -->
        <div v-if="filteredOptifine.length > 0" class="grid transition-all duration-300 ease-in-out" :style="{ gridTemplateRows: isExpanded('optifine') ? '1fr' : '0fr' }">
          <div class="overflow-hidden min-h-0">
            <div class="border-t border-gray-200 p-3 space-y-1.5 max-h-48 overflow-y-auto">
              <div
                v-for="ver in filteredOptifine"
                :key="ver.display_name"
                class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
                :class="selectedOptifine === ver.display_name
                  ? 'border-green-400 bg-green-50 shadow-sm'
                  : 'border-gray-200 hover:border-green-300 hover:bg-green-50/50'"
                @click="selectAndCollapse('optifine', ver.display_name, selectedOptifine)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm" :class="selectedOptifine === ver.display_name ? 'text-green-800 font-medium' : 'text-gray-700'">{{ ver.display_name }}</span>
                  <span v-if="ver.is_preview" class="text-xs px-1.5 py-0.5 rounded bg-yellow-100 text-yellow-700 font-medium">预览版</span>
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- 无版本时提示 -->
        <div v-else class="px-4 pb-3 text-xs text-gray-400">
          暂无适用于此版本的 OptiFine
        </div>
      </div>

      <!-- LiteLoader (仅 <= 1.12.2) -->
      <div
        v-if="showLiteloader"
        class="bg-white rounded-lg border overflow-hidden transition-colors"
        :class="isLoaderDisabled('liteloader') ? 'border-gray-200 opacity-60' : 'border-gray-300'"
      >
        <div
          class="flex items-center justify-between px-4 py-3 transition-colors"
          :class="isLoaderDisabled('liteloader') || liteloaderVersions.length === 0 ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
          @click="liteloaderVersions.length > 0 && !isLoaderDisabled('liteloader') && toggleCard('liteloader')"
        >
          <div class="flex items-center gap-2 min-w-0">
            <img :src="liteloaderIcon" class="w-5 h-5 rounded shrink-0" />
            <span class="text-sm font-medium text-gray-900 shrink-0">LiteLoader</span>
            <span v-if="selectedLiteloader" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-teal-100 text-teal-700 font-medium">
              {{ selectedLiteloader }}
              <button class="p-0.5 -mr-1 hover:bg-teal-200 rounded-full transition-colors" @click="clearSelection('liteloader', $event)">
                <XMarkIcon class="w-3 h-3" />
              </button>
            </span>
            <span v-if="isLoaderDisabled('liteloader')" class="text-xs text-red-500 ml-1">{{ getLoaderError('liteloader') }}</span>
          </div>
          <svg v-if="liteloaderVersions.length > 0" class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0" :class="{ 'rotate-180': isExpanded('liteloader') }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </div>
        <!-- 有版本时展开列表 -->
        <div v-if="liteloaderVersions.length > 0" class="grid transition-all duration-300 ease-in-out" :style="{ gridTemplateRows: isExpanded('liteloader') ? '1fr' : '0fr' }">
          <div class="overflow-hidden min-h-0">
            <div class="border-t border-gray-200 p-3 space-y-1.5 max-h-48 overflow-y-auto">
              <div
                v-for="ver in liteloaderVersions"
                :key="ver"
                class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
                :class="selectedLiteloader === ver
                  ? 'border-teal-400 bg-teal-50 shadow-sm'
                  : 'border-gray-200 hover:border-teal-300 hover:bg-teal-50/50'"
                @click="selectAndCollapse('liteloader', ver, selectedLiteloader)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm" :class="selectedLiteloader === ver ? 'text-teal-800 font-medium' : 'text-gray-700'">{{ ver }}</span>
                </div>
              </div>
              <div class="px-3 py-2 bg-teal-50 rounded-lg text-xs text-teal-700">
                LiteLoader 是一个轻量级的 Mod 加载器，专注于客户端 Mod，体积小、启动快。已于 1.12.2 停止更新，仅适用于旧版本。
              </div>
            </div>
          </div>
        </div>
        <!-- 无版本时提示 -->
        <div v-else class="px-4 pb-3 text-xs text-gray-400">
          暂无适用于此版本的 LiteLoader
        </div>
      </div>

      <!-- 无可用加载器 -->
      <div v-if="!showForge && !showNeoforge && !showFabric && !showLiteloader && !filteredOptifine.length" class="text-center py-12 text-gray-500">
        此版本暂无可用的加载器
      </div>
    </div>

    <!-- 底部 -->
    <div class="px-6 py-4 bg-white border-t border-gray-300 flex items-center justify-between shrink-0">
      <div class="text-sm text-gray-500">
        <span v-if="hasSelection">版本名: {{ getInstanceName() }}</span>
        <span v-else>不选择加载器 = 安装原版</span>
      </div>
      <button
        class="px-6 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="installing || !compatible"
        @click="handleInstall"
      >
        {{ installing ? '安装中...' : '开始安装' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.animate-slide-in {
  animation: slide-in 0.3s ease-out;
}

@keyframes slide-in {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
