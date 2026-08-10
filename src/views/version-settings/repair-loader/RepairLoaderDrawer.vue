<script setup lang="ts">
/**
 * 加载器检测/重装抽屉
 *
 * 打开后自动扫描（detectLoaderDamage），按结果展示：无损坏 → 绿色提示；
 * 损坏 → 询问是否重装，确认后调用 repairVersionLoader，监听后端
 * `repair-loader-progress` 事件按阶段渲染进度条：
 * 重装 → 合并资源文件 → 完成（遮蔽罩提示）/错误。
 */
import { ref, watch, computed, onUnmounted } from 'vue'
import {
  CheckCircleIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import Button from '@/components/common/Button.vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import * as tauri from '@/utils/tauri'
import {
  REPAIR_LOADER_PROGRESS_EVENT,
  type RepairLoaderProgress,
  type DetectLoaderResult,
} from '@/utils/api/personalization'

const props = defineProps<{
  visible: boolean
  versionId: string | null
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

type Phase = RepairLoaderProgress['phase'] | 'idle' | 'healthy' | 'confirm' | 'cancelled'

const phase = ref<Phase>('idle')
const scanProgress = ref(0)
const installProgress = ref(0)
const damaged = ref(false)
const repaired = ref(false)
const message = ref('')
const health = ref<DetectLoaderResult | null>(null)

let scanTimer: ReturnType<typeof setInterval> | null = null
let busy = false

onGlobalEvent<RepairLoaderProgress>(REPAIR_LOADER_PROGRESS_EVENT, (payload) => {
  if (payload.versionId !== props.versionId || payload.phase === 'scanning') return
  message.value = payload.message
  damaged.value = payload.damaged
  repaired.value = payload.repaired
  if (payload.phase === 'installing') {
    phase.value = 'installing'
    installProgress.value = payload.progress
  } else if (payload.phase === 'merging') {
    phase.value = 'merging'
    scanProgress.value = 100
    installProgress.value = 100
  } else if (payload.phase === 'done' || payload.phase === 'error') {
    phase.value = payload.phase
    scanProgress.value = 100
    installProgress.value = 100
  }
})

function clearScanTimer() {
  if (scanTimer) {
    clearInterval(scanTimer)
    scanTimer = null
  }
}

function reset() {
  phase.value = 'idle'
  scanProgress.value = 0
  installProgress.value = 0
  damaged.value = false
  repaired.value = false
  message.value = ''
  health.value = null
  clearScanTimer()
}

function isSupportedLoader(type: string | null): boolean {
  return type === 'forge' || type === 'neoforge' || type === 'fabric' || type === 'liteloader'
}

async function startScan() {
  if (!props.versionId || busy) return
  busy = true
  phase.value = 'scanning'
  scanProgress.value = 0
  scanTimer = setInterval(() => {
    scanProgress.value = Math.min(90, scanProgress.value + 4 + Math.random() * 9)
  }, 150)
  try {
    const result = await tauri.detectLoaderDamage(props.versionId)
    clearScanTimer()
    scanProgress.value = 100
    await new Promise((r) => setTimeout(r, 300))
    health.value = result
    damaged.value = !result.healthy
    if (result.healthy) {
      phase.value = 'healthy'
      message.value = result.reason || '当前文件无损坏'
    } else if (isSupportedLoader(result.loaderType)) {
      phase.value = 'confirm'
      message.value = result.reason
    } else {
      phase.value = 'error'
      message.value = result.reason || '该加载器暂不支持自动修复'
    }
  } catch (e) {
    clearScanTimer()
    phase.value = 'error'
    message.value = String(e)
  } finally {
    busy = false
  }
}

async function confirmRepair() {
  if (!props.versionId || busy) return
  busy = true
  phase.value = 'installing'
  installProgress.value = 0
  try {
    await tauri.repairVersionLoader(props.versionId)
  } catch (e) {
    if ((phase.value as Phase) !== 'error') {
      phase.value = 'error'
      message.value = String(e)
    }
  } finally {
    busy = false
  }
}

function cancelRepair() {
  clearScanTimer()
  phase.value = 'cancelled'
  message.value = '已取消重装，加载器仍处于损坏状态'
}

watch(
  () => props.visible,
  (open) => {
    if (!open) return
    if (phase.value === 'installing' || phase.value === 'merging') return
    reset()
    void startScan()
  },
)

onUnmounted(clearScanTimer)

const loaderDesc = computed(() => {
  const h = health.value
  if (!h || !h.loaderType) return '加载器文件'
  const names: Record<string, string> = {
    forge: 'Forge',
    neoforge: 'NeoForge',
    fabric: 'Fabric',
    liteloader: 'LiteLoader',
  }
  const type = names[h.loaderType] ?? h.loaderType
  return `${type} ${h.loaderVersion}`.trim() + (h.mcVersion ? `（MC ${h.mcVersion}）` : '')
})

const showScanBar = computed(() =>
  ['scanning', 'healthy', 'confirm', 'cancelled', 'installing', 'merging', 'done'].includes(phase.value),
)
const showInstallBar = computed(() => ['installing', 'merging', 'done'].includes(phase.value))
const showMerging = computed(() => phase.value === 'merging')
const completedHealthy = computed(
  () => phase.value === 'healthy' || (phase.value === 'done' && !damaged.value),
)
const completedFixed = computed(() => phase.value === 'done' && damaged.value && repaired.value)
const isError = computed(() => phase.value === 'error')
const isCancelled = computed(() => phase.value === 'cancelled')
</script>

<template>
  <Drawer
    :visible="visible"
    title="检测并重装加载器"
    placement="right"
    :width="420"
    render-in-place
    popup-container="#app-content"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="relative flex flex-1 flex-col px-6 py-5">
      <div v-if="phase === 'idle'" class="flex flex-1 flex-col items-center justify-center gap-3 text-gray-400">
        <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
        <p class="text-sm">正在准备检测...</p>
      </div>

      <template v-else>
        <div
          class="flex min-h-0 flex-1 flex-col gap-6 transition-all duration-300"
          :class="{ 'pointer-events-none blur-sm opacity-50': completedFixed }"
        >
          <div v-if="showScanBar" class="space-y-2">
            <div class="flex items-center justify-between text-xs">
              <span class="text-gray-500">扫描加载器</span>
              <span class="text-gray-400">{{ Math.round(scanProgress) }}%</span>
            </div>
            <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
              <div
                class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                :style="{ width: `${scanProgress}%` }"
              ></div>
            </div>
          </div>

          <div v-if="completedHealthy" class="flex flex-1 flex-col items-center justify-center text-gray-400">
            <CheckCircleIcon class="mb-2 h-8 w-8 text-green-400" />
            <span class="text-xs">{{ message }}</span>
          </div>

          <div v-if="phase === 'confirm'" class="rounded-xl border border-yellow-200 bg-yellow-50 p-4">
            <div class="flex items-start gap-3">
              <ExclamationTriangleIcon class="mt-0.5 h-5 w-5 shrink-0 text-yellow-500" />
              <div class="min-w-0 flex-1 space-y-1">
                <p class="text-sm font-semibold text-gray-800">检测到加载器损坏</p>
                <p class="break-all text-xs leading-relaxed text-gray-500">
                  {{ loaderDesc }} 文件缺失或已损坏，是否重新安装？
                </p>
              </div>
            </div>
            <div class="mt-4 flex justify-end gap-2">
              <Button type="outline" @click="cancelRepair">取消</Button>
              <Button type="primary" @click="confirmRepair">重新安装</Button>
            </div>
          </div>

          <div v-if="isCancelled" class="flex items-center gap-2 text-sm text-gray-500">
            <XCircleIcon class="h-5 w-5" />
            <span class="flex-1">{{ message }}</span>
            <Button size="small" type="outline" @click="startScan">重新检测</Button>
          </div>

          <div v-if="showInstallBar" class="space-y-2">
            <div class="flex items-center justify-between text-xs">
              <span class="text-gray-500">重新安装加载器</span>
              <span class="text-gray-400">{{ installProgress }}%</span>
            </div>
            <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
              <div
                class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                :style="{ width: `${installProgress}%` }"
              ></div>
            </div>
          </div>

          <div v-if="showMerging" class="flex items-center gap-2 text-sm text-gray-600">
            <svg class="h-4 w-4 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
            <span>正在合并资源文件，请稍后...</span>
          </div>

          <div v-if="isError" class="flex items-center gap-2 text-sm text-red-600">
            <XCircleIcon class="h-5 w-5" />
            <span class="break-all">{{ message }}</span>
          </div>
        </div>

        <div
          v-if="completedFixed"
          class="absolute inset-0 z-10 flex items-center justify-center bg-white/60 backdrop-blur-sm"
        >
          <div class="flex flex-col items-center gap-3 rounded-xl bg-white/95 px-8 py-6 shadow-lg">
            <CheckCircleIcon class="h-12 w-12 text-green-500" />
            <p class="text-sm font-semibold text-gray-800">检查到文件有损坏，已完成修复</p>
          </div>
        </div>
      </template>
    </div>
  </Drawer>
</template>

<style scoped>
:deep(.drawer-body) {
  display: flex;
  flex-direction: column;
  padding: 0;
  overflow: hidden;
}
</style>
