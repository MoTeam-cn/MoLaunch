<script setup lang="ts">
/**
 * 资源包/光影列表空状态组件
 * 三种 variant：loading（spinner）/ empty（未安装）/ no-match（筛选无匹配）
 */
import { CubeIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import type { PackKind } from '@/utils/tauri'

const props = defineProps<{
  variant: 'loading' | 'empty' | 'no-match'
  count: number
  kind: PackKind
}>()

defineEmits<{ install: [] }>()

const kindLabel = props.kind === 'resourcepack' ? '资源包' : '光影'
</script>

<template>
  <!-- 加载中（与 VersionSelect 统一样式） -->
  <div v-if="variant === 'loading'" class="flex h-full items-center justify-center">
    <div class="flex flex-col items-center gap-3 text-gray-400">
      <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
        <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <span class="text-sm">正在加载{{ kindLabel }}列表...</span>
    </div>
  </div>

  <!-- 空列表 / 无匹配 -->
  <div v-else class="flex h-full min-h-[400px] items-center justify-center">
    <div class="flex flex-col items-center text-center">
      <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-gray-100">
        <CubeIcon class="h-8 w-8 text-gray-300" />
      </div>
      <div class="mb-2 text-[15px] font-semibold text-gray-600">
        {{ count === 0 ? `尚未安装${kindLabel}` : '没有符合条件的项目' }}
      </div>
      <p v-if="count === 0" class="mb-5 text-[13px] text-gray-400">
        你可以从文件安装{{ kindLabel }}，或打开文件夹放入
      </p>
      <p v-else class="mb-5 text-[13px] text-gray-400">
        试试调整筛选条件或搜索关键词
      </p>
      <Button
        v-if="count === 0"
        type="primary"
        @click="$emit('install')"
      >
        从文件安装{{ kindLabel }}
      </Button>
    </div>
  </div>
</template>
