<script setup lang="ts">
/**
 * 资源包/光影列表空状态组件
 * 四种 variant：loading（spinner）/ empty（未安装）/ no-match（筛选无匹配）/ not-modable（光影无加载器）
 */
import { CubeIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import type { PackKind } from '@/utils/tauri'

const props = defineProps<{
  variant: 'loading' | 'empty' | 'no-match' | 'not-modable'
  count: number
  kind: PackKind
}>()

defineEmits<{
  install: []
  'go-download': []
  'go-select': []
}>()

const kindLabel = props.kind === 'resourcepack' ? '资源包' : '光影'
</script>

<template>
  <!-- 不可用（仅光影：无 OptiFine/Iris 加载器） -->
  <div v-if="variant === 'not-modable'" class="flex items-center justify-center py-12">
    <div class="rounded-xl border border-gray-200 bg-white p-8 text-center shadow-sm">
      <div class="mb-3 text-lg font-semibold text-gray-700">该版本不支持{{ kindLabel }}</div>
      <div class="mx-auto mb-5 h-0.5 w-12 bg-gray-300"></div>
      <p class="mb-5 text-sm text-gray-500">
        你需要先安装 OptiFine 或 Iris（配合 Fabric/Forge）才能使用光影，请在下载页面安装对应版本。<br>
        如果你已安装过，可能是版本选择有误，请切换版本。
      </p>
      <div class="flex justify-center gap-3">
        <Button
          type="primary"
          @click="$emit('go-download')"
        >
          转到下载页面
        </Button>
        <Button
          type="outline"
          @click="$emit('go-select')"
        >
          版本选择
        </Button>
      </div>
    </div>
  </div>

  <!-- 加载中（与 VersionSelect 统一样式） -->
  <div v-else-if="variant === 'loading'" class="flex h-full items-center justify-center">
    <div class="flex flex-col items-center gap-3 text-gray-400">
      <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
        <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <span class="text-sm">正在加载{{ kindLabel }}列表...</span>
    </div>
  </div>

  <!-- 空列表 / 无匹配 -->
  <div v-else class="flex h-full items-center justify-center">
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
