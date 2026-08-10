<script setup lang="ts">
/**
 * 加载器检测/重装进度抽屉
 *
 * 打开后触发 `repairVersionLoader`，监听后端 `repair-loader-progress` 事件按阶段
 * 渲染进度条：扫描 →（损坏时）重装 → 合并资源文件 → 完成/错误。
 */
import { ref, watch, computed } from 'vue'
import { CheckCircleIcon, XCircleIcon } from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import { REPAIR_LOADER_PROGRESS_EVENT, type RepairLoaderProgress } from '@/utils/api/personalization'

const props = defineProps<{
  visible: boolean
  versionId: string | null
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

type Phase = RepairLoaderProgress['phase'] | 'idle'

const phase = ref<Phase>('idle')
const scanProgress = ref(0)
const installProgress = ref(0)
const damaged = ref(false)
const repaired = ref(false)
const message = ref('')

onGlobalEvent<RepairLoaderProgress>(REPAIR_LOADER_PROGRESS_EVENT, (payload) => {
  if (payload.versionId !== props.versionId) return
  phase.value = payload.phase
  if (payload.phase === 'scanning') scanProgress.value = payload.progress
  if (payload.phase === 'installing') installProgress.value = payload.progress
  if (payload.phase === 'merging' || payload.phase === 'done') {
    scanProgress.value = 100
    installProgress.value = 100
  }
  damaged.value = payload.damaged
  repaired.value = payload.repaired
  message.value = payload.message
})

watch(
  () => props.visible,
  (open) => {
    if (open) {
      phase.value = 'idle'
      scanProgress.value = 0
      installProgress.value = 0
      damaged.value = false
      repaired.value = false
      message.value = ''
    }
  },
)

const showScanBar = computed(() =>
  ['scanning', 'installing', 'merging', 'done'].includes(phase.value),
)
const showInstallBar = computed(() => ['installing', 'merging', 'done'].includes(phase.value))
const showMerging = computed(() => phase.value === 'merging')
const completedHealthy = computed(() => phase.value === 'done' && !damaged.value)
const completedFixed = computed(() => phase.value === 'done' && damaged.value && repaired.value)
const isError = computed(() => phase.value === 'error')
</script>

<template>
  <Drawer
    :visible="visible"
    title="检测并重装加载器"
    placement="right"
    :width="420"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="flex h-full flex-col px-6 py-5">
      <div v-if="phase === 'idle'" class="flex flex-1 flex-col items-center justify-center gap-3 text-gray-400">
        <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
        <p class="text-sm">正在准备检测...</p>
      </div>

      <template v-else>
        <div class="relative flex flex-1 flex-col">
          <div
            class="flex flex-1 flex-col gap-6 transition-all duration-300"
            :class="{ 'pointer-events-none blur-sm opacity-50': completedFixed }"
          >
            <div v-if="showScanBar" class="space-y-2">
              <div class="flex items-center justify-between text-xs">
                <span class="text-gray-500">扫描加载器</span>
                <span class="text-gray-400">{{ scanProgress }}%</span>
              </div>
              <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                  :style="{ width: `${scanProgress}%` }"
                ></div>
              </div>
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

            <div v-if="completedHealthy" class="flex items-center gap-2 text-sm text-green-600">
              <CheckCircleIcon class="h-5 w-5" />
              <span>{{ message || '当前文件无损坏' }}</span>
            </div>

            <div v-if="isError" class="flex items-center gap-2 text-sm text-red-600">
              <XCircleIcon class="h-5 w-5" />
              <span>{{ message }}</span>
            </div>
          </div>

          <div
            v-if="completedFixed"
            class="absolute inset-0 z-10 flex items-center justify-center bg-white/50 backdrop-blur-sm"
          >
            <div class="flex flex-col items-center gap-3 rounded-xl bg-white/90 px-8 py-6 shadow-lg">
              <CheckCircleIcon class="h-12 w-12 text-green-500" />
              <p class="text-sm font-semibold text-gray-800">检查到文件有损坏，已完成修复</p>
            </div>
          </div>
        </div>
      </template>
    </div>
  </Drawer>
</template>
