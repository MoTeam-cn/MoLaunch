<script setup lang="ts">
/**
 * Fabric API 信息卡片
 *
 * 选择 Fabric Loader 后显示，告知用户将自动安装哪个版本的 Fabric API。
 * 后端在 install_merged 时已自动安装最新版，此处仅做信息展示。
 *
 * 父组件通过 useFabricApi composable 管理状态，将结果以 props 传入本组件。
 */
import { ArrowPathIcon } from '@heroicons/vue/24/outline'
import Alert from '@/components/common/Alert.vue'
import Button from '@/components/common/Button.vue'
import { formatBytes, formatDate } from '@/utils/format'
import type { FabricApiVersion } from '@/utils/api/loader'
import type { FabricApiState } from '@/composables/useFabricApi'
import fabricIcon from '@/assets/blocks/Fabric.png'

interface Props {
  mcVersion: string
  state: FabricApiState
  latest: FabricApiVersion | null
  error: string
}

defineProps<Props>()
defineEmits<{ retry: [] }>()
</script>

<template>
  <div class="bg-white rounded-lg border border-blue-200 overflow-hidden ml-4">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between px-4 py-2.5 bg-blue-50/40">
      <div class="flex items-center gap-2 min-w-0">
        <img :src="fabricIcon" class="w-4 h-4 rounded shrink-0 opacity-80" />
        <span class="text-sm font-medium text-gray-900 shrink-0">Fabric API</span>
        <span class="text-xs px-2 py-0.5 rounded-full font-medium bg-blue-100 text-blue-700">
          将自动安装
        </span>
      </div>
      <Button
        v-if="state === 'error'"
        type="text"
        size="mini"
        class="shrink-0"
        @click="$emit('retry')"
      >
        <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
        重试
      </Button>
    </div>

    <!-- 内容区 -->
    <div class="px-4 py-3 border-t border-blue-100">
      <!-- Loading -->
      <div v-if="state === 'loading'" class="flex items-center gap-2 text-xs text-gray-500">
        <svg class="animate-spin w-4 h-4 text-blue-500 shrink-0" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        正在获取 Fabric API 版本信息...
      </div>

      <!-- Error -->
      <Alert
        v-else-if="state === 'error'"
        type="warning"
        :message="`获取 Fabric API 版本信息失败：${error}`"
        :truncate="false"
      />

      <!-- Empty -->
      <div v-else-if="state === 'empty'" class="text-xs text-gray-500">
        未找到适用于 Minecraft {{ mcVersion }} 的 Fabric API 版本
      </div>

      <!-- Success: 展示最新版本信息 -->
      <div v-else-if="state === 'success' && latest" class="space-y-1.5">
        <div class="flex items-center gap-2">
          <span class="text-xs text-gray-500 shrink-0 w-14">版本号</span>
          <span class="text-sm font-medium text-gray-900">{{ latest.version_number }}</span>
        </div>
        <div class="flex items-center gap-2 min-w-0">
          <span class="text-xs text-gray-500 shrink-0 w-14">文件名</span>
          <span class="text-xs text-gray-700 truncate" :title="latest.file_name">{{ latest.file_name }}</span>
        </div>
        <div class="flex items-center gap-2 text-xs text-gray-500">
          <span class="shrink-0 w-14">发布日期</span>
          <span>{{ formatDate(latest.release_date) }}</span>
          <span class="text-gray-300">·</span>
          <span>{{ formatBytes(latest.size, 1) }}</span>
        </div>
        <p class="text-xs text-blue-600 pt-1 leading-relaxed">
          安装时将自动下载此版本，安装完成后可在 Mod 管理页面手动更换版本
        </p>
      </div>
    </div>
  </div>
</template>
