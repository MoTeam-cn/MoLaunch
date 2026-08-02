<script setup lang="ts">
/**
 * Java 下载按钮 + 进度条
 * 当 javaReqs 存在且需要下载时显示"下载 Java X"按钮，点击后从 Mojang 官方 Runtime 索引下载
 * 下载完成后 emit('downloaded', javaPath)，由父组件刷新 Java 列表并选中
 */
import { ref, onMounted, computed } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import Button from '@/components/common/Button.vue'
import type { JavaRequirements, JavaDownloadProgress } from '@/types/java'

const props = defineProps<{ javaReqs: JavaRequirements | null }>()
const emit = defineEmits<{ downloaded: [javaPath: string] }>()

const downloading = ref(false)
const progress = ref<JavaDownloadProgress | null>(null)

const { start } = useTauriEvent<JavaDownloadProgress>(
  tauri.JAVA_DOWNLOAD_PROGRESS_EVENT,
  (payload) => { progress.value = payload },
)

/** 要下载的 Java 大版本号 */
const targetMajor = computed(() => {
  if (!props.javaReqs) return 0
  return props.javaReqs.recommended_java_version || props.javaReqs.min_java_version || 0
})

/** 进度百分比（0~100） */
const progressPercent = computed(() => {
  const p = progress.value
  if (!p) return 0
  if (p.total > 0) return Math.min(100, Math.round((p.current / p.total) * 100))
  if (p.bytes_total > 0) return Math.min(100, Math.round((p.bytes_downloaded / p.bytes_total) * 100))
  return p.stage === 'done' ? 100 : 30 // 不确定阶段显示 30%
})

async function handleDownload() {
  if (!targetMajor.value || downloading.value) return
  downloading.value = true
  progress.value = null
  try {
    const javaPath = await tauri.downloadJava(targetMajor.value)
    toastSuccess(`Java ${targetMajor.value} 下载完成`)
    emit('downloaded', javaPath)
  } catch (e) {
    toastError('Java 下载失败：' + String(e))
  } finally {
    downloading.value = false
    progress.value = null
  }
}

onMounted(() => {
  start()
})
</script>

<template>
  <div v-if="javaReqs && targetMajor" class="space-y-1.5">
    <!-- 下载按钮（不在下载中时显示） -->
    <Button
      v-if="!downloading"
      type="primary"
      size="small"
      @click="handleDownload"
    >
      <template #icon>
        <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10 3a1 1 0 011 1v6.586l2.293-2.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 111.414-1.414L9 10.586V4a1 1 0 011-1z" />
          <path d="M3 14a1 1 0 011 1v1h12v-1a1 1 0 112 0v2a1 1 0 01-1 1H3a1 1 0 01-1-1v-2a1 1 0 011-1z" />
        </svg>
      </template>
      下载 Java {{ targetMajor }}
    </Button>

    <!-- 下载进度条 -->
    <div v-if="downloading && progress" class="rounded-md bg-blue-50 px-3 py-2 text-xs text-blue-700">
      <div class="mb-1 flex items-center justify-between gap-2">
        <span class="truncate">{{ progress.message || '下载中...' }}</span>
        <span class="flex-none font-medium">{{ progressPercent }}%</span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-blue-200">
        <div
          class="h-full bg-blue-600 transition-all duration-200"
          :style="{ width: progressPercent + '%' }"
        />
      </div>
      <div v-if="progress.total > 0" class="mt-1 text-[10px] text-blue-500">
        {{ progress.current }} / {{ progress.total }} 文件
      </div>
    </div>

    <!-- 下载中但还没收到进度事件 -->
    <div v-else-if="downloading" class="rounded-md bg-blue-50 px-3 py-2 text-xs text-blue-700">
      正在准备下载...
    </div>
  </div>
</template>
