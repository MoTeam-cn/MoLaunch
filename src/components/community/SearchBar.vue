<script setup lang="ts">
/**
 * 搜索筛选栏（参考 PCL2 PageResource 搜索区）
 * 两行布局：名称+来源 / 版本+加载器+类型
 * 使用项目自定义 Select 组件
 */

import { ref, watch, computed } from 'vue'
import type { ResourceType, CategoryTagInfo } from '@/types/community'
import { SOURCE_OPTIONS, LOADER_OPTIONS } from '@/types/community'
import Select from '@/components/common/Select.vue'
import { MagnifyingGlassIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
import { getCategoryTags } from '@/utils/api/community'

const props = defineProps<{
  query: string
  resourceType: ResourceType
  gameVersion: string
  modLoader: number
  source: number
  category: string
}>()
const emit = defineEmits<{
  'update:query': [v: string]
  'update:gameVersion': [v: string]
  'update:modLoader': [v: number]
  'update:source': [v: number]
  'update:category': [v: string]
  search: []
  reset: []
}>()

const categories = ref<CategoryTagInfo[]>([])

/** 资源类型变化时自动加载分类标签 */
watch(
  () => props.resourceType,
  async (rt) => {
    try {
      categories.value = await getCategoryTags(rt)
    } catch {
      categories.value = []
    }
  },
  { immediate: true },
)

/** Select 组件选项 */
const sourceOptions = computed(() => SOURCE_OPTIONS.map(o => ({ label: o.label, value: o.value })))
const loaderOptions = computed(() => LOADER_OPTIONS.map(o => ({ label: o.label, value: o.value })))
const categoryOptions = computed(() => [
  { label: '全部', value: '' },
  ...categories.value.map(c => ({ label: c.label, value: c.combined })),
])

/** 常用游戏版本 */
const commonVersions = [
  '1.21.4', '1.21.1', '1.20.1', '1.19.2', '1.18.2',
  '1.16.5', '1.12.2', '1.7.10',
]

/** 搜索框防抖 */
let timer: ReturnType<typeof setTimeout> | null = null
const localQuery = ref(props.query)
watch(() => props.query, (v) => { localQuery.value = v })

function onInput() {
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => {
    emit('update:query', localQuery.value)
    emit('search')
  }, 500)
}
</script>

<template>
  <div class="space-y-2.5">
    <!-- 第一行：名称 + 来源 -->
    <div class="grid grid-cols-[1fr_auto_auto] gap-3 items-center">
      <div class="relative">
        <MagnifyingGlassIcon class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          v-model="localQuery"
          type="text"
          placeholder="搜索资源名称..."
          class="w-full pl-9 pr-3 py-2 text-sm bg-white border border-gray-300 rounded-lg outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
          @input="onInput"
          @keyup.enter="emit('search')"
        >
      </div>
      <div class="w-36">
        <Select
          :model-value="source"
          :options="sourceOptions"
          @update:model-value="emit('update:source', $event as number)"
        />
      </div>
      <button
        class="px-3 py-2 rounded-lg text-sm text-gray-500 border border-gray-300 hover:bg-gray-50 transition-colors flex items-center gap-1"
        @click="emit('reset')"
      >
        <ArrowPathIcon class="w-4 h-4" />
        重置
      </button>
    </div>

    <!-- 第二行：版本 + 加载器 + 类型 -->
    <div class="grid grid-cols-3 gap-3">
      <!-- 游戏版本 -->
      <div class="relative">
        <input
          :value="gameVersion"
          @input="emit('update:gameVersion', ($event.target as HTMLInputElement).value)"
          type="text"
          list="common-versions"
          placeholder="游戏版本"
          class="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg outline-none focus:border-primary-500"
        >
        <datalist id="common-versions">
          <option v-for="v in commonVersions" :key="v" :value="v" />
        </datalist>
      </div>

      <!-- 加载器 -->
      <Select
        :model-value="modLoader"
        :options="loaderOptions"
        @update:model-value="emit('update:modLoader', $event as number)"
      />

      <!-- 分类 -->
      <Select
        :model-value="category"
        :options="categoryOptions"
        @update:model-value="emit('update:category', $event as string)"
      />
    </div>
  </div>
</template>
