<script setup lang="ts">
/**
 * 搜索筛选栏
 * 两行布局：名称+来源 / 版本+加载器+类型
 * 使用项目自定义 Select 组件
 */

import { ref, watch, computed } from 'vue'
import type { ResourceType, CategoryTagInfo } from '@/types/community'
import { SOURCE_OPTIONS, LOADER_OPTIONS } from '@/types/community'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import { MagnifyingGlassIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
import { getCategoryTags } from '@/utils/api/community'
import { useSearchHistory } from '@/composables/useSearchHistory'

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

/** 搜索历史（localStorage 持久化，最近 5 条） */
const { history: searchHistory, add: addHistory } = useSearchHistory()
const showHistory = ref(false)

function onFocus() {
  if (searchHistory.value.length > 0) {
    showHistory.value = true
  }
}

function onBlur() {
  // 延迟隐藏，让历史项 mousedown 先触发
  setTimeout(() => { showHistory.value = false }, 150)
}

function selectHistory(term: string) {
  localQuery.value = term
  emit('update:query', term)
  showHistory.value = false
  commitSearch()
}

/** 主动搜索（回车/搜索按钮/点击历史）：记录历史 + 触发搜索 */
function commitSearch() {
  const term = localQuery.value.trim()
  if (term) addHistory(term)
  emit('search')
}

/** Select 选项变化时同步更新并立即触发搜索 */
function selectAndUpdate(field: 'source' | 'modLoader' | 'category', value: number | string) {
  emit(`update:${field}` as any, value)
  emit('search')
}
</script>

<template>
  <div class="space-y-2.5">
    <!-- 第一行：名称 + 来源 + 搜索按钮 -->
    <div class="grid grid-cols-[1fr_auto_auto_auto] gap-3 items-center">
      <div class="relative">
        <Input
          v-model="localQuery"
          placeholder="搜索资源名称..."
          class="w-full"
          @input="onInput"
          @focus="onFocus"
          @blur="onBlur"
          @keydown.enter="commitSearch"
        >
          <template #prefix>
            <MagnifyingGlassIcon class="w-4 h-4 text-gray-400" />
          </template>
        </Input>
        <!-- 搜索历史下拉（focus 时展示，点击历史项填充并搜索） -->
        <div
          v-if="showHistory && searchHistory.length > 0"
          class="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-md shadow-lg z-20 overflow-hidden"
        >
          <div
            v-for="term in searchHistory"
            :key="term"
            class="px-3 py-1.5 text-sm text-gray-700 hover:bg-primary-50 hover:text-primary-700 cursor-pointer flex items-center gap-2 transition-colors"
            @mousedown.prevent="selectHistory(term)"
          >
            <MagnifyingGlassIcon class="w-3.5 h-3.5 text-gray-400 shrink-0" />
            <span class="truncate flex-1">{{ term }}</span>
          </div>
        </div>
      </div>
      <div class="w-36">
        <Select
          :model-value="source"
          :options="sourceOptions"
          @update:model-value="selectAndUpdate('source', $event as number)"
        />
      </div>
      <Button type="primary" size="small" @click="commitSearch">
        <template #icon>
          <MagnifyingGlassIcon class="w-4 h-4" />
        </template>
        搜索
      </Button>
      <Button type="outline" size="small" @click="emit('reset')">
        <template #icon>
          <ArrowPathIcon class="w-4 h-4" />
        </template>
        重置
      </Button>
    </div>

    <!-- 第二行：版本 + 加载器 + 类型 -->
    <div class="grid grid-cols-3 gap-3">
      <!-- 游戏版本 -->
      <Input
        :model-value="gameVersion"
        placeholder="游戏版本"
        list="common-versions"
        @update:model-value="emit('update:gameVersion', $event)"
        @input="emit('update:gameVersion', $event)"
        @keydown.enter="emit('search')"
      />
      <datalist id="common-versions">
        <option v-for="v in commonVersions" :key="v" :value="v" />
      </datalist>

      <!-- 加载器 -->
      <Select
        :model-value="modLoader"
        :options="loaderOptions"
        @update:model-value="selectAndUpdate('modLoader', $event as number)"
      />

      <!-- 分类 -->
      <Select
        :model-value="category"
        :options="categoryOptions"
        @update:model-value="selectAndUpdate('category', $event as string)"
      />
    </div>
  </div>
</template>
