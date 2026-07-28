<script setup lang="ts">
/**
 * 整合包要求卡片（联机大厅阶段 4 新增）
 *
 * 加入方拉取房间详情后，若房主关联了整合包（`roomState.hostModpack` 非空）则渲染此组件。
 *
 * 三态：
 * - **已安装**（绿色）：本地已装同款整合包（manifest_hash 或三元组匹配），可直接加入房间
 * - **可安装**（蓝色）：本地未安装，展示整合包信息 + 一键安装按钮
 * - **不可安装**（红色）：来源平台不支持 / 校验失败，提示用户手动处理
 *
 * # 安全设计
 * 不从房主接收 `downloadUrl`。一键安装时通过 `getProjectVersions` 反查平台 API
 * 获取下载链接，避免 api-server / 房主成为 URL 分发中心（详见 lobby-modpack-share.md）。
 */
import { ref, computed, onMounted } from 'vue'
import { checkLocalModpack } from '@/utils/api/version'
import { useModpackInstall } from '@/composables/useModpackInstall'
import { formatBytes } from '@/utils/format'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import type { ModpackMeta } from '@/types/online'
import {
  CheckCircleIcon,
  ExclamationTriangleIcon,
  ArrowPathIcon,
  ArrowDownTrayIcon,
  CubeIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 房主上报的整合包元数据（必传，父组件已确定有整合包才渲染） */
  modpack: ModpackMeta
}>()

type CheckState = 'loading' | 'installed' | 'notInstalled' | 'error'
const checkState = ref<CheckState>('loading')
const installedVersionId = ref<string | undefined>(undefined)
const errorMsg = ref('')

const { installing, install } = useModpackInstall()

const sourceLabel = computed(() => {
  const s = props.modpack.source
  if (s === 'curseforge') return 'CurseForge'
  if (s === 'modrinth') return 'Modrinth'
  return s
})

/** 是否可一键安装（仅 curseforge / modrinth） */
const canInstall = computed(() =>
  props.modpack.source === 'curseforge' || props.modpack.source === 'modrinth',
)

async function runCheck() {
  checkState.value = 'loading'
  errorMsg.value = ''
  try {
    const result = await checkLocalModpack(
      props.modpack.manifestHash,
      props.modpack.source,
      props.modpack.projectId,
      props.modpack.fileId,
    )
    if (result.installed) {
      installedVersionId.value = result.versionId
      checkState.value = 'installed'
    } else {
      checkState.value = 'notInstalled'
    }
  } catch (e) {
    errorMsg.value = e instanceof Error ? e.message : String(e)
    checkState.value = 'error'
  }
}

async function handleInstall() {
  const ok = await install(props.modpack)
  if (ok) {
    // 安装触发即跳转下载页，组件随后会被父组件卸载，无需额外处理
    return
  }
}

onMounted(() => {
  void runCheck()
})
</script>

<template>
  <div class="rounded-lg border border-gray-200 overflow-hidden">
    <!-- 标题栏 -->
    <div class="px-4 py-2.5 bg-gray-50 border-b border-gray-200 flex items-center gap-2">
      <CubeIcon class="w-4 h-4 text-gray-500 shrink-0" />
      <span class="text-sm font-medium text-gray-700">整合包要求</span>
      <span
        v-if="checkState === 'installed'"
        class="ml-auto inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-green-100 text-green-700"
      >
        已安装
      </span>
      <span
        v-else-if="checkState === 'notInstalled' && canInstall"
        class="ml-auto inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-700"
      >
        需安装
      </span>
      <span
        v-else-if="checkState === 'notInstalled' && !canInstall"
        class="ml-auto inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-red-100 text-red-700"
      >
        不可安装
      </span>
      <span
        v-else-if="checkState === 'error'"
        class="ml-auto inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-red-100 text-red-700"
      >
        校验失败
      </span>
      <span
        v-else
        class="ml-auto inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-gray-100 text-gray-500"
      >
        校验中
      </span>
    </div>

    <!-- 整合包信息（所有状态共用） -->
    <div class="px-4 py-3 space-y-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-gray-900 truncate">{{ modpack.name }}</span>
        <span class="px-1.5 py-0.5 text-xs rounded bg-primary-100 text-primary-700 shrink-0">
          {{ sourceLabel }}
        </span>
      </div>
      <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-600">
        <span v-if="modpack.modpackVersion">版本: {{ modpack.modpackVersion }}</span>
        <span v-if="modpack.loader">加载器: {{ modpack.loader }}{{ modpack.loaderVersion ? ' ' + modpack.loaderVersion : '' }}</span>
        <span>MC: {{ modpack.mcVersion }}</span>
        <span v-if="modpack.fileCount">Mods: {{ modpack.fileCount }}</span>
        <span v-if="modpack.fileSize">大小: {{ formatBytes(modpack.fileSize) }}</span>
      </div>

      <!-- 状态提示区 -->
      <!-- 已安装 -->
      <div v-if="checkState === 'installed'" class="mt-2 p-2.5 bg-green-50 rounded text-xs text-green-700 flex items-start gap-1.5">
        <CheckCircleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <div class="flex-1">
          <div>本地已安装同款整合包，可直接加入房间</div>
          <div v-if="installedVersionId" class="mt-0.5 text-green-600">
            版本 ID: <code class="bg-white px-1 py-0.5 rounded">{{ installedVersionId }}</code>
          </div>
        </div>
      </div>

      <!-- 可安装 -->
      <div v-else-if="checkState === 'notInstalled' && canInstall" class="mt-2 p-2.5 bg-blue-50 rounded text-xs text-blue-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <div class="flex-1">
          <div>本地未安装此整合包，点击下方按钮一键安装</div>
          <div class="mt-1.5">
            <Button type="primary" size="small" :loading="installing" @click="handleInstall">
              <template #icon><ArrowDownTrayIcon class="w-3.5 h-3.5" /></template>
              一键安装
            </Button>
          </div>
        </div>
      </div>

      <!-- 不可安装（来源不支持） -->
      <div v-else-if="checkState === 'notInstalled' && !canInstall" class="mt-2 p-2.5 bg-red-50 rounded text-xs text-red-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <div class="flex-1">
          <div>不支持的整合包来源：{{ modpack.source }}</div>
          <div class="mt-0.5 text-red-600">请手动安装后再加入房间</div>
        </div>
      </div>

      <!-- 校验失败 -->
      <div v-else-if="checkState === 'error'" class="mt-2 p-2.5 bg-red-50 rounded text-xs text-red-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <div class="flex-1">
          <div>校验本地整合包失败：{{ errorMsg }}</div>
          <div class="mt-1.5">
            <Tooltip text="重新校验本地是否已安装">
              <Button type="ghost" size="small" @click="runCheck">
                <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
                重试
              </Button>
            </Tooltip>
          </div>
        </div>
      </div>

      <!-- 校验中 -->
      <div v-else class="mt-2 p-2.5 bg-gray-50 rounded text-xs text-gray-500 flex items-center gap-1.5">
        <ArrowPathIcon class="w-3.5 h-3.5 animate-spin" />
        <span>正在校验本地是否已安装...</span>
      </div>
    </div>
  </div>
</template>
