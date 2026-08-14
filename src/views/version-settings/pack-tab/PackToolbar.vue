<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 资源包/光影管理顶部工具栏
 * - 从文件安装 / 打开文件夹 / 刷新
 * - 全部/已启用/已禁用 筛选按钮组（带计数 badge）
 * - 搜索框
 */
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import {
  ArrowDownTrayIcon,
  FolderOpenIcon,
  ArrowPathIcon,
  MagnifyingGlassIcon,
} from '@heroicons/vue/24/outline'
import type { PackKind } from '@/utils/tauri'

const props = defineProps<{
  packsLoading: boolean
  filterOptions: Array<{ v: 'all' | 'enabled' | 'disabled'; l: string; count: number }>
  kind: PackKind
}>()

const packFilter = defineModel<'all' | 'enabled' | 'disabled'>('packFilter', { required: true })
const packSearch = defineModel<string>('packSearch', { required: true })

defineEmits<{
  install: []
  'open-dir': []
  refresh: []
}>()

const kindLabel = props.kind === 'resourcepack' ? '资源包' : '光影'
</script>

<template>
  <!-- 顶部工具栏（flex-none 固定，不随列表滚动） -->
  <section class="flex-none border-b border-gray-200 bg-white px-6 py-3">
    <div class="flex items-center gap-2">
      <div class="flex items-center gap-2">
        <Tooltip :text="`从本地 zip 文件安装${kindLabel}`" position="bottom">
          <Button
            type="primary"
            size="small"
            @click="$emit('install')"
          >
            <template #icon><ArrowDownTrayIcon class="h-3.5 w-3.5" /></template>
            从文件安装
          </Button>
        </Tooltip>
        <Tooltip :text="`在系统资源管理器中打开${kindLabel}目录`" position="bottom">
          <Button
            type="outline"
            size="small"
            @click="$emit('open-dir')"
          >
            <template #icon><FolderOpenIcon class="h-3.5 w-3.5" /></template>
            打开文件夹
          </Button>
        </Tooltip>
        <Tooltip :text="`重新扫描${kindLabel}目录`" position="bottom">
          <Button
            type="outline"
            size="small"
            :loading="packsLoading"
            @click="$emit('refresh')"
          >
            <template #icon><ArrowPathIcon class="h-3.5 w-3.5" /></template>
            刷新
          </Button>
        </Tooltip>
      </div>

      <div class="ml-auto flex items-center gap-2">
        <div class="flex flex-shrink-0 items-center gap-1.5 rounded-lg bg-gray-100 p-1">
          <button
            v-for="opt in filterOptions"
            :key="opt.v"
            class="flex items-center gap-1.5 whitespace-nowrap rounded-md px-3 py-1 text-xs font-medium transition-colors"
            :class="packFilter === opt.v
              ? 'bg-white text-primary-700 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'"
            @click="packFilter = opt.v"
          >
            {{ opt.l }}
            <span
              class="whitespace-nowrap rounded-full px-1.5 py-0.5 text-[10px] tabular-nums leading-none"
              :class="packFilter === opt.v
                ? 'bg-primary-100 text-primary-700'
                : 'bg-gray-200 text-gray-500'"
            >{{ opt.count }}</span>
          </button>
        </div>

        <Input
          v-model="packSearch"
          :placeholder="`搜索${kindLabel}名称`"
          size="small"
          width="210px"
        >
          <template #prefix><MagnifyingGlassIcon class="h-3.5 w-3.5" /></template>
        </Input>
      </div>
    </div>
  </section>
</template>
