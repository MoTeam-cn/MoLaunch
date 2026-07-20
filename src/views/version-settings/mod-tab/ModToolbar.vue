<script setup lang="ts">
/**
 * Mod 管理顶部工具栏
 * - 从文件安装 / 打开文件夹 / 刷新
 * - 全部/已启用/已禁用 筛选按钮组（带计数 badge）
 * - 搜索框
 */
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import {
  ArrowDownTrayIcon,
  FolderOpenIcon,
  ArrowPathIcon,
  MagnifyingGlassIcon,
} from '@heroicons/vue/24/outline'

defineProps<{
  modsLoading: boolean
  filterOptions: Array<{ v: 'all' | 'enabled' | 'disabled'; l: string; count: number }>
}>()

const modFilter = defineModel<'all' | 'enabled' | 'disabled'>('modFilter', { required: true })
const modSearch = defineModel<string>('modSearch', { required: true })

defineEmits<{
  install: []
  'open-dir': []
  refresh: []
}>()
</script>

<template>
  <!-- 顶部工具栏（flex-none 固定，不随列表滚动） -->
  <section class="flex-none border-b border-gray-200 bg-white px-6 py-3">
    <div class="flex flex-wrap items-center gap-2">
      <Tooltip text="从本地 jar 文件安装 Mod" position="bottom">
        <Button
          type="primary"
          size="small"
          @click="$emit('install')"
        >
          <template #icon><ArrowDownTrayIcon class="h-3.5 w-3.5" /></template>
          从文件安装
        </Button>
      </Tooltip>
      <Tooltip text="在系统资源管理器中打开 mods 目录" position="bottom">
        <Button
          type="outline"
          size="small"
          @click="$emit('open-dir')"
        >
          <template #icon><FolderOpenIcon class="h-3.5 w-3.5" /></template>
          打开文件夹
        </Button>
      </Tooltip>
      <Tooltip text="重新扫描 mods 目录" position="bottom">
        <Button
          type="outline"
          size="small"
          :loading="modsLoading"
          @click="$emit('refresh')"
        >
          <template #icon><ArrowPathIcon class="h-3.5 w-3.5" /></template>
          刷新
        </Button>
      </Tooltip>

      <div class="ml-auto flex items-center gap-1 rounded-lg bg-gray-100 p-0.5">
        <button
          v-for="opt in filterOptions"
          :key="opt.v"
          class="flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors"
          :class="modFilter === opt.v
            ? 'bg-white text-primary-700 shadow-sm'
            : 'text-gray-500 hover:text-gray-700'"
          @click="modFilter = opt.v"
        >
          {{ opt.l }}
          <span
            class="rounded-full px-1.5 py-0.5 text-[10px] leading-none"
            :class="modFilter === opt.v
              ? 'bg-primary-100 text-primary-700'
              : 'bg-gray-200 text-gray-500'"
          >{{ opt.count }}</span>
        </button>
      </div>

      <Input
        v-model="modSearch"
        placeholder="搜索 Mod 名称"
        size="small"
        class="w-56"
      >
        <template #prefix><MagnifyingGlassIcon class="h-3.5 w-3.5" /></template>
      </Input>
    </div>
  </section>
</template>
