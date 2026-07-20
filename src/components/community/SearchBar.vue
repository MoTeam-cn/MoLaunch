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
      <Input
        v-model="localQuery"
        placeholder="搜索资源名称..."
        class="w-full"
        @input="onInput"
        @keydown.enter="emit('search')"
      >
        <template #prefix>
          <MagnifyingGlassIcon class="w-4 h-4 text-gray-400" />
        </template>
      </Input>
      <div class="w-36">
        <Select
          :model-value="source"
          :options="sourceOptions"
          @update:model-value="emit('update:source', $event as number)"
        />
      </div>
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
      />
      <datalist id="common-versions">
        <option v-for="v in commonVersions" :key="v" :value="v" />
      </datalist>

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
