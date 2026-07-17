<script setup lang="ts">
/**
 * Mod 列表空状态组件
 *
 * 四种 variant：
 * - not-modable：该版本不可使用 Mod（显示跳转下载/版本选择按钮）
 * - loading：正在加载 Mod 列表（spinner）
 * - empty：未安装任何 Mod（按 modsCount===0 判断，显示安装按钮）
 * - no-match：有 Mod 但筛选/搜索后无匹配（仅提示文案）
 */
import { PuzzlePieceIcon } from '@heroicons/vue/24/outline'

defineProps<{
  variant: 'not-modable' | 'loading' | 'empty' | 'no-match'
  modsCount: number
}>()

defineEmits<{
  'go-download': []
  'go-select': []
  install: []
}>()
</script>

<template>
  <!-- 不可安装 Mod 的提示 -->
  <div v-if="variant === 'not-modable'" class="flex items-center justify-center py-12">
    <div class="rounded-xl border border-gray-200 bg-white p-8 text-center shadow-sm">
      <div class="mb-3 text-lg font-semibold text-gray-700">该版本不可使用 Mod</div>
      <div class="mx-auto mb-5 h-0.5 w-12 bg-gray-300"></div>
      <p class="mb-5 text-sm text-gray-500">
        你需要先安装 Forge、Fabric 等 Mod 加载器才能使用 Mod，请在下载页面安装这些版本。<br>
        如果你已经安装过 Mod 加载器，可能是版本选择有误，请切换版本。
      </p>
      <div class="flex justify-center gap-3">
        <button
          class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
          @click="$emit('go-download')"
        >
          转到下载页面
        </button>
        <button
          class="rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50"
          @click="$emit('go-select')"
        >
          版本选择
        </button>
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
      <span class="text-sm">正在加载 Mod 列表...</span>
    </div>
  </div>

  <!-- 空列表 / 无匹配 -->
  <div v-else class="flex items-center justify-center py-12">
    <div class="text-center">
      <PuzzlePieceIcon class="mx-auto mb-3 h-10 w-10 text-gray-300" />
      <div class="mb-2 text-base font-medium text-gray-500">
        {{ modsCount === 0 ? '尚未安装 Mod' : '没有符合条件的 Mod' }}
      </div>
      <p v-if="modsCount === 0" class="mb-4 text-xs text-gray-400">
        你可以从文件安装 Mod，或下载新 Mod
      </p>
      <button
        v-if="modsCount === 0"
        class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
        @click="$emit('install')"
      >
        从文件安装 Mod
      </button>
    </div>
  </div>
</template>
