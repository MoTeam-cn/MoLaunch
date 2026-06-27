<script setup lang="ts">
/**
 * 加载器选择组件
 */

import { ref, onMounted, computed } from 'vue'
import * as tauri from '@/utils/tauri'
import { ChevronLeftIcon } from '@heroicons/vue/24/outline'
import LoaderCard from '@/components/common/LoaderCard.vue'
import Tooltip from '@/components/common/Tooltip.vue'

import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

interface Props {
  mcVersion: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  back: []
  install: [options: { mcVersion: string; forge?: string; neoforge?: string; fabric?: string; optifine?: string; liteloader?: string; instanceName: string }]
}>()

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

const mcNum = computed(() => {
  const parts = props.mcVersion.split('.')
  return (parseInt(parts[0]) || 0) * 10000 + (parseInt(parts[1]) || 0) * 100 + (parseInt(parts[2]) || 0)
})

const showForge = computed(() => mcNum.value >= 10501)
const showNeoforge = computed(() => mcNum.value >= 12001)
const showFabric = computed(() => mcNum.value > 11300)
const showLiteloader = computed(() => mcNum.value <= 11202)

// 过滤 OptiFine：只显示与当前 MC 版本匹配的
const filteredOptifine = computed(() => {
  return optifineVersions.value.filter(v => {
    const match = v.display_name.match(/^([\d.]+)\s/)
    if (!match) return false
    return match[1] === props.mcVersion
  })
})

// 构建版本列表
const forgeItems = computed(() =>
  forgeVersions.value.map((v, i) => ({
    key: v,
    label: v,
    tags: i === 0 ? ['最新版'] : [],
  }))
)

const neoforgeItems = computed(() =>
  [...neoforgeVersions.value].reverse().map(v => ({
    key: v.version,
    label: `${props.mcVersion}.${v.version}`,
    tags: v.recommended ? ['推荐'] : [],
  }))
)

const fabricItems = computed(() =>
  fabricVersions.value.map(v => ({
    key: v.version,
    label: v.version,
    tags: [v.stable ? '稳定版' : '测试版'],
  }))
)

const optifineItems = computed(() =>
  filteredOptifine.value.map(v => ({
    key: v.display_name,
    label: v.display_name,
    tags: v.is_preview ? ['预览版'] : [],
  }))
)

const liteloaderItems = computed(() =>
  liteloaderVersions.value.map(v => ({
    key: v,
    label: v,
  }))
)

// 兼容性检查
function getLoaderError(loader: string): string | null {
  if (loader === 'forge') {
    if (selectedFabric.value) return '与 Fabric 不兼容'
    if (selectedNeoforge.value) return '与 NeoForge 不兼容'
  }
  if (loader === 'neoforge') {
    if (selectedForge.value) return '与 Forge 不兼容'
    if (selectedFabric.value) return '与 Fabric 不兼容'
    if (selectedOptifine.value) return '与 OptiFine 不兼容'
  }
  if (loader === 'fabric') {
    if (selectedForge.value) return '与 Forge 不兼容'
    if (selectedNeoforge.value) return '与 NeoForge 不兼容'
  }
  if (loader === 'optifine') {
    if (selectedNeoforge.value) return '与 NeoForge 不兼容'
  }
  return null
}

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

function getInstanceName(): string {
  let name = props.mcVersion
  if (selectedFabric.value) name += `-Fabric${selectedFabric.value}`
  if (selectedForge.value) name += `-Forge_${selectedForge.value}`
  if (selectedNeoforge.value) name += `-NeoForge_${selectedNeoforge.value}`
  if (selectedOptifine.value) name += `-OptiFine`
  if (selectedLiteloader.value) name += `-LiteLoader_${selectedLiteloader.value}`
  return name
}

const hasSelection = computed(() =>
  selectedForge.value || selectedNeoforge.value || selectedFabric.value || selectedOptifine.value || selectedLiteloader.value
)

onMounted(async () => {
  loading.value = true
  const tasks: Promise<void>[] = []
  if (showForge.value) {
    tasks.push(tauri.listForgeVersions(props.mcVersion).then(v => { forgeVersions.value = v }).catch(() => {}))
  }
  if (showNeoforge.value) {
    tasks.push(tauri.listNeoforgeVersions(props.mcVersion).then(v => { neoforgeVersions.value = v }).catch(() => {}))
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

function handleInstall() {
  emit('install', {
    mcVersion: props.mcVersion,
    forge: selectedForge.value || undefined,
    neoforge: selectedNeoforge.value || undefined,
    fabric: selectedFabric.value || undefined,
    optifine: selectedOptifine.value || undefined,
    liteloader: selectedLiteloader.value || undefined,
    instanceName: getInstanceName(),
  })
}
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
      <LoaderCard
        v-if="showForge && forgeVersions.length > 0"
        id="forge"
        name="Forge"
        :icon="anvilIcon"
        color="orange"
        description="经典的 Mod 加载器，拥有最丰富的 Mod 生态，适合大多数 Mod 包。"
        :versions="forgeItems"
        :selected="selectedForge"
        :disabled="isLoaderDisabled('forge')"
        :disabled-reason="getLoaderError('forge') || ''"
        @select="v => selectedForge = v"
        @clear="selectedForge = null"
      />

      <LoaderCard
        v-if="showNeoforge && neoforgeVersions.length > 0"
        id="neoforge"
        name="NeoForge"
        :icon="neoforgeIcon"
        color="purple"
        description="Forge 的社区分支，更新更快，适配新版 Minecraft，是 Forge 的现代替代方案。"
        :versions="neoforgeItems"
        :selected="selectedNeoforge"
        :disabled="isLoaderDisabled('neoforge')"
        :disabled-reason="getLoaderError('neoforge') || ''"
        @select="v => selectedNeoforge = v"
        @clear="selectedNeoforge = null"
      />

      <LoaderCard
        v-if="showFabric && fabricVersions.length > 0"
        id="fabric"
        name="Fabric"
        :icon="fabricIcon"
        color="blue"
        description="轻量级现代 Mod 加载器，启动快、更新及时，适合客户端 Mod 和性能优化类 Mod。"
        :versions="fabricItems"
        :selected="selectedFabric"
        :disabled="isLoaderDisabled('fabric')"
        :disabled-reason="getLoaderError('fabric') || ''"
        @select="v => selectedFabric = v"
        @clear="selectedFabric = null"
      />

      <LoaderCard
        id="optifine"
        name="OptiFine"
        :icon="optifineIcon"
        color="green"
        description="性能优化与光影 Mod，提升帧数、支持 shader，可与 Forge/Fabric 共存。"
        :versions="optifineItems"
        :selected="selectedOptifine"
        :disabled="isLoaderDisabled('optifine')"
        :disabled-reason="getLoaderError('optifine') || ''"
        :show-versions="filteredOptifine.length > 0"
        @select="v => selectedOptifine = v"
        @clear="selectedOptifine = null"
      />

      <LoaderCard
        v-if="showLiteloader"
        id="liteloader"
        name="LiteLoader"
        :icon="liteloaderIcon"
        color="teal"
        description="轻量级 Mod 加载器，专注于客户端 Mod，体积小、启动快。已于 1.12.2 停止更新。"
        :versions="liteloaderItems"
        :selected="selectedLiteloader"
        :show-versions="liteloaderVersions.length > 0"
        @select="v => selectedLiteloader = v"
        @clear="selectedLiteloader = null"
      />
    </div>

    <!-- 底部 -->
    <div class="px-6 py-4 bg-white border-t border-gray-300 flex items-center justify-between shrink-0">
      <div class="text-sm text-gray-500">
        <span v-if="hasSelection">版本名: {{ getInstanceName() }}</span>
        <span v-else>不选择加载器 = 安装原版</span>
      </div>
      <Tooltip v-if="loading" text="当前页面状态未加载完成，请等待加载" position="top">
        <button
          class="px-6 py-2 text-sm font-medium text-white bg-gray-400 rounded-lg cursor-not-allowed"
        >
          开始安装
        </button>
      </Tooltip>
      <button
        v-else
        class="px-6 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
        @click="handleInstall"
      >
        开始安装
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
