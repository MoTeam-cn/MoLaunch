<script setup lang="ts">
/**
 * 加载器选择组件
 *
 * 子模块：
 *   - useLoaderData composable：负责获取 5 种加载器版本列表 + 缓存 + computed 版本项
 *
 * 本组件保留：
 *   - MC 版本类型判断（snapshot/fool/ancient + showXxx 标志）
 *   - 选中状态管理 + 兼容性检查
 *   - 实例名生成 + 安装按钮
 */

import { ref, onMounted, computed } from 'vue'
import { useVersionStore } from '@/stores/version'
import { ChevronLeftIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/outline'
import LoaderCard from '@/components/common/LoaderCard.vue'
import Alert from '@/components/common/Alert.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import FabricApiInfoCard from '@/components/install/FabricApiInfoCard.vue'
import { useLoaderData } from '@/composables/useLoaderData'
import { useFabricApi } from '@/composables/useFabricApi'

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

const versionStore = useVersionStore()

// 选中状态
const selectedForge = ref<string | null>(null)
const selectedNeoforge = ref<string | null>(null)
const selectedFabric = ref<string | null>(null)
const selectedOptifine = ref<string | null>(null)
const selectedLiteloader = ref<string | null>(null)

// 自定义版本名称
const customInstanceName = ref('')
const showNameInput = ref(false)

// —— MC 版本类型判断 ——
const mcNum = computed(() => {
  const parts = props.mcVersion.split('.')
  return (parseInt(parts[0]) || 0) * 10000 + (parseInt(parts[1]) || 0) * 100 + (parseInt(parts[2]) || 0)
})

const versionInfo = computed(() => versionStore.getVersionById(props.mcVersion))
const isSnapshot = computed(() => versionInfo.value?.version_type === 'snapshot')
const isFool = computed(() => versionInfo.value?.version_type === 'fool')
const isAncient = computed(() => {
  const type = versionInfo.value?.version_type
  return type === 'old_beta' || type === 'old_alpha' || mcNum.value < 10000
})

const showForge = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value >= 10501)
const showNeoforge = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value >= 12001)
const showFabric = computed(() => !isAncient.value && !isFool.value && mcNum.value > 11300)
const showLiteloader = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value <= 11202)
const showOptifine = computed(() => !isAncient.value && !isFool.value)

// 加载器版本数据（获取 + 缓存 + computed 版本项）
const {
  forgeItems, neoforgeItems, fabricItems, optifineItems, liteloaderItems,
  loadingForge, loadingNeoforge, loadingFabric, loadingOptifine, loadingLiteloader,
  fetchAll,
} = useLoaderData(computed(() => props.mcVersion), {
  forge: showForge,
  neoforge: showNeoforge,
  fabric: showFabric,
  optifine: showOptifine,
  liteloader: showLiteloader,
})

// —— Fabric API 信息 ——
// 后端在 install_merged 时已自动安装最新版 Fabric API，
// 此处仅做信息展示：告知用户将自动安装哪个版本，便于了解。
// 状态管理 + 查询逻辑抽到 useFabricApi composable，展示抽到 FabricApiInfoCard 子组件
const {
  fabricApiState,
  fabricApiLatest,
  fabricApiError,
  retry: retryFabricApi,
} = useFabricApi(computed(() => props.mcVersion), selectedFabric)

// —— 兼容性检查 ——
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

// —— 实例名 ——
function getDefaultInstanceName(): string {
  let name = props.mcVersion
  if (selectedFabric.value) name += `-Fabric${selectedFabric.value}`
  if (selectedForge.value) name += `-Forge_${selectedForge.value}`
  if (selectedNeoforge.value) name += `-NeoForge_${selectedNeoforge.value}`
  if (selectedOptifine.value) name += `-OptiFine`
  if (selectedLiteloader.value) name += `-LiteLoader_${selectedLiteloader.value}`
  return name
}

const instanceName = computed(() => customInstanceName.value || getDefaultInstanceName())

const hasSelection = computed(() =>
  selectedForge.value || selectedNeoforge.value || selectedFabric.value || selectedOptifine.value || selectedLiteloader.value
)

function handleInstall() {
  emit('install', {
    mcVersion: props.mcVersion,
    forge: selectedForge.value || undefined,
    neoforge: selectedNeoforge.value || undefined,
    fabric: selectedFabric.value || undefined,
    optifine: selectedOptifine.value || undefined,
    liteloader: selectedLiteloader.value || undefined,
    instanceName: instanceName.value,
  })
}

onMounted(() => {
  fetchAll()
})
</script>

<template>
  <div class="flex flex-col h-full animate-slide-in">
    <!-- 顶栏 -->
    <div class="px-6 py-4 bg-white border-b border-gray-300 flex items-center gap-3 shrink-0">
      <Button type="ghost" size="small" @click="emit('back')">
        <template #icon><ChevronLeftIcon class="w-5 h-5 text-gray-600" /></template>
      </Button>
      <div>
        <h2 class="text-lg font-semibold text-gray-900">选择加载器</h2>
        <p class="text-xs text-gray-500">Minecraft {{ mcVersion }} — 可选安装 Mod 加载器</p>
      </div>
    </div>

    <!-- 加载器列表 -->
    <div class="flex-1 overflow-y-auto p-6 space-y-3">
      <Alert v-if="isAncient" type="warning" message="版本过老，无任何配套内容，请直接安装原版即可" />
      <Alert v-if="isSnapshot" type="info" message="快照版建议选择 Fabric，Forge/NeoForge 一般不会为快照适配" />
      <Alert v-if="isFool" type="info" message="愚人节版本不支持 Mod 加载器，请直接安装原版体验" />

      <!-- Forge -->
      <LoaderCard
        v-if="showForge"
        id="forge" name="Forge" :icon="anvilIcon" color="orange"
        description="经典的 Mod 加载器，拥有最丰富的 Mod 生态，适合大多数 Mod 包。"
        :versions="loadingForge ? [] : forgeItems"
        :selected="selectedForge"
        :disabled="isLoaderDisabled('forge') || loadingForge"
        :disabled-reason="loadingForge ? '获取 Forge 版本中... 先别急哈！' : getLoaderError('forge') || ''"
        :loading="loadingForge"
        @select="v => selectedForge = v"
        @clear="selectedForge = null"
      />

      <!-- NeoForge -->
      <LoaderCard
        v-if="showNeoforge"
        id="neoforge" name="NeoForge" :icon="neoforgeIcon" color="purple"
        description="Forge 的社区分支，更新更快，适配新版 Minecraft，是 Forge 的现代替代方案。"
        :versions="loadingNeoforge ? [] : neoforgeItems"
        :selected="selectedNeoforge"
        :disabled="isLoaderDisabled('neoforge') || loadingNeoforge"
        :disabled-reason="loadingNeoforge ? '获取 NeoForge 版本中... 先别急哈！' : getLoaderError('neoforge') || ''"
        :loading="loadingNeoforge"
        @select="v => selectedNeoforge = v"
        @clear="selectedNeoforge = null"
      />

      <!-- Fabric -->
      <LoaderCard
        v-if="showFabric"
        id="fabric" name="Fabric" :icon="fabricIcon" color="blue"
        description="轻量级现代 Mod 加载器，启动快、更新及时，适合客户端 Mod 和性能优化类 Mod。"
        :versions="loadingFabric ? [] : fabricItems"
        :selected="selectedFabric"
        :disabled="isLoaderDisabled('fabric') || loadingFabric"
        :disabled-reason="loadingFabric ? '获取 Fabric 版本中... 先别急哈！' : getLoaderError('fabric') || ''"
        :loading="loadingFabric"
        @select="v => selectedFabric = v"
        @clear="selectedFabric = null"
      />

      <!-- Fabric API 信息卡片（选择 Fabric 后显示，仅信息展示，后端自动安装最新版） -->
      <FabricApiInfoCard
        v-if="selectedFabric"
        :mc-version="mcVersion"
        :state="fabricApiState"
        :latest="fabricApiLatest"
        :error="fabricApiError"
        @retry="retryFabricApi"
      />

      <!-- OptiFine -->
      <LoaderCard
        v-if="showOptifine"
        id="optifine" name="OptiFine" :icon="optifineIcon" color="green"
        description="性能优化与光影 Mod，提升帧数、支持 shader，可与 Forge/Fabric 共存。"
        :versions="loadingOptifine ? [] : optifineItems"
        :selected="selectedOptifine"
        :disabled="isLoaderDisabled('optifine') || loadingOptifine"
        :disabled-reason="loadingOptifine ? '获取 OptiFine 版本中... 先别急哈！' : getLoaderError('optifine') || ''"
        :loading="loadingOptifine"
        @select="v => selectedOptifine = v"
        @clear="selectedOptifine = null"
      />

      <!-- LiteLoader -->
      <LoaderCard
        v-if="showLiteloader"
        id="liteloader" name="LiteLoader" :icon="liteloaderIcon" color="teal"
        description="轻量级 Mod 加载器，专注于客户端 Mod，体积小、启动快。已于 1.12.2 停止更新。"
        :versions="loadingLiteloader ? [] : liteloaderItems"
        :selected="selectedLiteloader"
        :disabled="loadingLiteloader"
        :disabled-reason="loadingLiteloader ? '获取 LiteLoader 版本中... 先别急哈！' : ''"
        :loading="loadingLiteloader"
        @select="v => selectedLiteloader = v"
        @clear="selectedLiteloader = null"
      />
    </div>

    <!-- 底部 -->
    <div class="px-6 py-4 bg-white border-t border-gray-300 shrink-0">
      <div class="flex items-center gap-2 mb-3">
        <span class="text-xs text-gray-500 shrink-0">版本名:</span>
        <div class="flex-1 min-w-0">
          <Input
            v-if="showNameInput"
            v-model="customInstanceName"
            :placeholder="getDefaultInstanceName()"
            size="small"
          />
          <span v-else class="text-sm font-medium text-gray-900">{{ instanceName }}</span>
        </div>
        <Button
          type="text"
          size="mini"
          @click="showNameInput = !showNameInput; if (showNameInput) customInstanceName = getDefaultInstanceName(); else customInstanceName = ''"
        >
          {{ showNameInput ? '使用默认' : '自定义' }}
        </Button>
      </div>
      <div class="flex items-center justify-between">
        <span v-if="!hasSelection" class="text-xs text-gray-400">不选择加载器 = 安装原版</span>
        <span v-else class="text-xs text-gray-400">&nbsp;</span>
        <Button
          type="primary"
          @click="handleInstall"
        >
          <template #icon><ArrowDownTrayIcon class="w-4 h-4" /></template>
          开始安装
        </Button>
      </div>
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
