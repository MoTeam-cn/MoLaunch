<script setup lang="ts">
/**
 * 通用导航侧边栏组件
 *
 * 功能：
 * - 渲染分类菜单（图标 + 标签），选中态高亮
 * - 切换菜单时更新 URL query 的 `tab` 参数（router.replace，不产生历史记录）
 * - 初始化时从 URL query `tab` 恢复选中项（刷新页面保留打开的分类）
 *
 * 用法：
 *   <NavSidebar v-model="activeCategory" :categories="categories" />
 *
 * 路由同步说明：
 * - 切换菜单 → router.replace({ query: { ...route.query, tab: id } })
 * - 页面加载 → 读取 route.query.tab，若有效则 emit('update:modelValue', tab)
 * - 保留其他 query 参数（如 VersionSettings 的 id），不冲突
 */
import { watch, onMounted, type Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'

interface NavCategory {
  id: string
  label: string
  icon: Component
  desc?: string
}

const props = defineProps<{
  modelValue: string
  categories: NavCategory[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', id: string): void
}>()

const route = useRoute()
const router = useRouter()

// 页面加载时从 URL query.tab 恢复选中项（刷新页面保留路径）
onMounted(() => {
  const tab = route.query.tab as string | undefined
  if (tab && tab !== props.modelValue && props.categories.some(c => c.id === tab)) {
    emit('update:modelValue', tab)
  }
})

// 选中项变化时同步到 URL query（不产生历史记录，保留其他 query 参数）
watch(() => props.modelValue, (val) => {
  const currentTab = route.query.tab as string | undefined
  if (val !== currentTab) {
    router.replace({ query: { ...route.query, tab: val } })
  }
})
</script>

<template>
  <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
    <div class="flex-1 overflow-y-auto py-4">
      <button
        v-for="cat in categories"
        :key="cat.id"
        type="button"
        class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors cursor-pointer"
        :class="[
          modelValue === cat.id
            ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
            : 'text-gray-700 hover:bg-gray-50',
        ]"
        @click="emit('update:modelValue', cat.id)"
      >
        <component :is="cat.icon" class="w-5 h-5 mr-3" />
        {{ cat.label }}
      </button>
    </div>
  </aside>
</template>
