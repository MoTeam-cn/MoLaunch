<script setup lang="ts">
/**
 * 整合包选择器（联机大厅阶段 3 新增）
 *
 * 创建房间时关联本地已安装整合包，自动读取 `versions/{id}/modpack.meta.json`。
 * - 复选框开启后调用 `readLocalModpackMeta` 读取元数据
 * - 有元数据：展示整合包信息卡（名称 / 来源 / 版本 / 加载器 / 文件数 / 大小）
 * - 无元数据：置灰提示「该版本无整合包元数据」（非平台安装或原版）
 * - versionId 变化时若已开启则自动重新读取
 *
 * # 复用约定
 * - checkbox 沿用项目惯例（`accent-primary-500`），与 WhitelistEditor / ExportTab 一致
 * - 文件大小格式化复用 `utils/format.ts` 的 `formatBytes`
 */
import { ref, computed, watch } from 'vue'
import { readLocalModpackMeta } from '@/utils/api/version'
import { formatBytes } from '@/utils/format'
import Tooltip from '@/components/common/Tooltip.vue'
import type { ModpackMeta, ModpackMetaFile } from '@/types/online'
import {
  CubeIcon,
  ArchiveBoxXMarkIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 当前选中的版本 ID（用于读取 modpack.meta.json） */
  versionId: string
  /** v-model 绑定值 */
  modelValue?: ModpackMeta
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ModpackMeta | undefined]
  /** 勾选状态变化（用于父组件徽章联动，即使版本无元数据也能反映勾选意图） */
  'enabled-change': [enabled: boolean]
}>()

/** 是否关联整合包 */
const enabled = ref(false)
/** 读取到的本地整合包元数据 */
const localMeta = ref<ModpackMetaFile | null>(null)
/** 读取中 */
const loading = ref(false)
/** 读取失败错误信息 */
const error = ref('')

/** 来源平台显示名 */
const sourceLabel = computed(() => {
  const s = localMeta.value?.source ?? ''
  if (s === 'curseforge') return 'CurseForge'
  if (s === 'modrinth') return 'Modrinth'
  return s
})

/** 未选版本时 Tooltip 提示文案（已选版本时为空，Tooltip 不显示） */
const tooltipText = computed(() => !props.versionId ? '请先选择 MC 版本' : '')

/** ModpackMetaFile → ModpackMeta（剥离 installedAt） */
function toMeta(m: ModpackMetaFile): ModpackMeta {
  return {
    source: m.source,
    projectId: m.projectId,
    fileId: m.fileId,
    mcVersion: m.mcVersion,
    modpackVersion: m.modpackVersion,
    name: m.name,
    loader: m.loader,
    loaderVersion: m.loaderVersion,
    fileSize: m.fileSize,
    fileCount: m.fileCount,
    manifestHash: m.manifestHash,
  }
}

/** 读取本地整合包元数据 */
async function loadMeta() {
  if (!props.versionId) {
    error.value = '请先选择 MC 版本'
    return
  }
  loading.value = true
  error.value = ''
  try {
    localMeta.value = await readLocalModpackMeta(props.versionId)
    emit('update:modelValue', localMeta.value ? toMeta(localMeta.value) : undefined)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    localMeta.value = null
    emit('update:modelValue', undefined)
  } finally {
    loading.value = false
  }
}

/** 切换启用状态 */
function onToggle(e: Event) {
  const target = e.target as HTMLInputElement
  const checked = target.checked
  // 未选版本时不允许勾选，强制恢复未勾选状态（Tooltip 会提示原因）
  if (checked && !props.versionId) {
    target.checked = false
    return
  }
  enabled.value = checked
  emit('enabled-change', checked)
  if (!checked) {
    localMeta.value = null
    error.value = ''
    emit('update:modelValue', undefined)
    return
  }
  void loadMeta()
}

/** versionId 变化时：若已启用则重新读取；清空时重置 */
watch(
  () => props.versionId,
  (v) => {
    if (!v) {
      localMeta.value = null
      error.value = ''
      if (enabled.value) emit('update:modelValue', undefined)
      return
    }
    if (enabled.value) void loadMeta()
  },
)
</script>

<template>
  <div class="space-y-2.5">
    <!-- 启用开关（未选版本时灰色 + Tooltip 提示，不再 disabled 以允许 hover 触发） -->
    <Tooltip :text="tooltipText" position="top" block>
      <label
        class="flex items-center gap-2 w-full"
        :class="versionId ? 'cursor-pointer' : 'cursor-not-allowed opacity-60'"
      >
        <input
          :checked="enabled"
          type="checkbox"
          class="accent-primary-500"
          @change="onToggle"
        />
        <CubeIcon class="w-4 h-4" :class="enabled ? 'text-primary-600' : 'text-gray-400'" />
        <span class="text-sm text-gray-800">关联整合包</span>
        <span class="text-xs text-gray-400">
          {{ enabled ? '已关联本地整合包元数据' : '上报给加入方以便校验或一键安装' }}
        </span>
      </label>
    </Tooltip>

    <!-- 读取中 -->
    <div
      v-if="enabled && loading"
      class="modpack-fade-in flex items-center gap-2 px-3 py-2 bg-gray-50 rounded text-xs text-gray-500"
    >
      <ArrowPathIcon class="w-3.5 h-3.5 animate-spin" />
      <span>正在读取整合包元数据...</span>
    </div>

    <!-- 错误提示 -->
    <div
      v-else-if="enabled && error"
      class="modpack-fade-in px-3 py-2 bg-red-50 rounded text-xs text-red-600"
    >
      {{ error }}
    </div>

    <!-- 整合包信息卡 -->
    <div
      v-else-if="enabled && localMeta"
      class="modpack-fade-in px-3 py-2.5 bg-primary-50/40 rounded border border-primary-100 space-y-1.5"
    >
      <div class="flex items-center gap-2">
        <CubeIcon class="w-4 h-4 text-primary-600 shrink-0" />
        <span class="text-sm font-medium text-gray-900 truncate">{{ localMeta.name }}</span>
        <span
          class="px-1.5 py-0.5 text-xs rounded bg-primary-100 text-primary-700 shrink-0"
        >
          {{ sourceLabel }}
        </span>
      </div>
      <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-600 pl-6">
        <span v-if="localMeta.modpackVersion">版本: {{ localMeta.modpackVersion }}</span>
        <span v-if="localMeta.loader">加载器: {{ localMeta.loader }}{{ localMeta.loaderVersion ? ' ' + localMeta.loaderVersion : '' }}</span>
        <span>MC: {{ localMeta.mcVersion }}</span>
        <span v-if="localMeta.fileCount">Mods: {{ localMeta.fileCount }}</span>
        <span v-if="localMeta.fileSize">大小: {{ formatBytes(localMeta.fileSize) }}</span>
      </div>
    </div>

    <!-- 空状态：无整合包元数据（icon + text 垂直水平居中） -->
    <div
      v-else-if="enabled && !localMeta && !loading && !error"
      class="modpack-fade-in py-5 flex flex-col items-center justify-center gap-2 text-gray-400 bg-gray-50 rounded"
    >
      <ArchiveBoxXMarkIcon class="w-6 h-6" />
      <span class="text-xs">该版本无整合包元数据（非平台安装或原版）</span>
    </div>
  </div>
</template>

<style scoped>
/* 提示框 fade-in 动画（避免突然蹦出显得僵硬） */
.modpack-fade-in {
  animation: modpack-fade-in 0.3s ease-out;
}
@keyframes modpack-fade-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
