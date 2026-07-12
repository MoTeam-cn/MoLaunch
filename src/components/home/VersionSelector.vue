<script setup lang="ts">
/**
 * 版本选择器组件
 * 基于通用 Select 组件，显示已安装版本列表
 */

import { ref, computed, onMounted, watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import Select from '@/components/common/Select.vue'
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'
import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

const versionStore = useVersionStore()

interface InstalledVersion {
  id: string
  version_type: string
}

const installed = ref<InstalledVersion[]>([])
const loading = ref(false)

/** 当前选中的版本 ID（从 store 同步） */
const selectedId = computed({
  get: () => versionStore.selectedVersion,
  set: (val) => { versionStore.selectedVersion = val }
})

/** 当前选中的版本对象 */
const selectedVersion = computed(() =>
  installed.value.find((v) => v.id === selectedId.value)
)

/** 当前选中版本的类型元数据（缓存，避免模板重复调用） */
const selectedTypeMeta = computed<TypeMeta>(() => {
  if (!selectedVersion.value) return defaultMeta
  return typeMeta(inferVersionType(selectedVersion.value.id, selectedVersion.value.version_type))
})

/**
 * 推断版本类型
 * modloader 字符串匹配优先（后端 version_type 对 forge 版本通常只返回 release），
 * 与 InstalledList.vue#inferVersionType 保持一致
 */
function inferVersionType(id: string, backendType: string): string {
  const lower = id.toLowerCase()
  if (lower.includes('neoforge')) return 'neoforge'
  if (lower.includes('forge')) return 'forge'
  if (lower.includes('fabric')) return 'fabric'
  if (lower.includes('optifine')) return 'optifine'
  if (lower.includes('liteloader')) return 'liteloader'
  if (/^\d{2}w\d{2}[a-z]/.test(id)) return 'snapshot'
  if (backendType) return backendType
  return 'release'
}

/** Select 组件的 options（type 为推断后的类型） */
const options = computed(() =>
  installed.value.map(v => ({
    label: v.id,
    value: v.id,
    type: inferVersionType(v.id, v.version_type),
  }))
)

/** 版本类型 → 方块图标 + 标签（与下载页/加载器选择页一致） */
interface TypeMeta {
  icon: string
  label: string
}
const typeMetaMap: Record<string, TypeMeta> = {
  release: { icon: grassIcon, label: '正式版' },
  snapshot: { icon: commandBlockIcon, label: '快照' },
  old_beta: { icon: cobblestoneIcon, label: '旧版' },
  old_alpha: { icon: cobblestoneIcon, label: '旧版' },
  fool: { icon: goldBlockIcon, label: '愚人节版' },
  forge: { icon: anvilIcon, label: 'Forge' },
  fabric: { icon: fabricIcon, label: 'Fabric' },
  neoforge: { icon: neoforgeIcon, label: 'NeoForge' },
  optifine: { icon: optifineIcon, label: 'OptiFine' },
  liteloader: { icon: liteloaderIcon, label: 'LiteLoader' },
}
const defaultMeta: TypeMeta = { icon: grassIcon, label: '其他' }

function typeMeta(type: string): TypeMeta {
  return typeMetaMap[type] ?? defaultMeta
}

async function loadInstalled() {
  loading.value = true
  try {
    installed.value = await tauri.listInstalledVersionsWithType()
    if (installed.value.length > 0) {
      const exists = installed.value.some((v) => v.id === selectedId.value)
      if (!exists) selectedId.value = installed.value[0].id
    }
  } catch (e) {
    console.error('Failed to load installed versions:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadInstalled()
})

// 监听下载完成时刷新列表
watch(() => versionStore.downloading, (val) => {
  if (!val) setTimeout(loadInstalled, 500)
})

defineExpose({ refresh: loadInstalled })
</script>

<template>
  <Select
    class="w-full"
    :model-value="selectedId"
    :options="options"
    @update:model-value="selectedId = String($event)"
  >
    <!-- 自定义触发器 -->
    <template #trigger="{ open, toggle }">
      <button
        class="flex w-full items-center justify-between rounded-lg border bg-white px-3 py-2.5 transition-colors disabled:cursor-not-allowed"
        :class="open ? 'border-primary-400 bg-primary-50/30' : 'border-gray-200 hover:border-primary-300 hover:bg-primary-50/30'"
        :disabled="loading"
        @click="toggle"
      >
        <div class="flex items-center gap-2 overflow-hidden">
          <span v-if="loading" class="text-sm text-gray-400">加载中...</span>
          <template v-else-if="selectedVersion">
            <!-- 版本类型方块图标（与下载页一致） -->
            <img
              :src="selectedTypeMeta.icon"
              :title="selectedTypeMeta.label"
              class="h-5 w-5 flex-none rounded"
              alt=""
            >
            <span class="truncate text-sm font-medium text-gray-900">{{ selectedVersion.id }}</span>
          </template>
          <span v-else class="text-sm text-gray-400">无可用版本</span>
        </div>
        <svg class="h-4 w-4 flex-none text-gray-400 transition-transform" :class="{ 'rotate-180': open }" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M5.3 7.3a1 1 0 011.4 0L10 10.6l3.3-3.3a1 1 0 111.4 1.4l-4 4a1 1 0 01-1.4 0l-4-4a1 1 0 010-1.4z" clip-rule="evenodd" />
        </svg>
      </button>
    </template>

    <!-- 自定义选项渲染 -->
    <template #option="{ option }">
      <!-- 版本类型方块图标（与下载页一致） -->
      <img
        :src="typeMeta(option.type).icon"
        :title="typeMeta(option.type).label"
        class="h-5 w-5 flex-none rounded"
        alt=""
      >
      <span class="flex-1 truncate text-sm text-gray-900">{{ option.label }}</span>
    </template>

    <!-- 空状态 -->
    <template #empty>暂无已安装版本</template>
  </Select>
</template>
