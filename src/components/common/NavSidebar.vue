<script setup lang="ts">
/**
 * 通用导航侧边栏组件
 *
 * 功能：
 * - 渲染分类菜单（图标 + 标签），选中态高亮
 * - 支持二级子菜单（children 字段，可选）：父项点击展开/收起，子项点击切换
 * - 子菜单展开/收起带动画（grid-template-rows 0fr→1fr + 图标旋转 + 透明度）
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
 *
 * 向后兼容：categories 项的 children 字段可选，未传时行为与原版一致（无子菜单）
 */
import { ref, watch, computed, type Component } from 'vue'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'
import { useTabPersistence } from '@/composables/useTabPersistence'

interface NavCategory {
  id: string
  label: string
  icon: Component
  desc?: string
  /** 可选子菜单项（有 children 时父项点击展开/收起，不切换选中态） */
  children?: NavCategory[]
  /** 可选禁用态：灰色不可点击，用于「房间详情」等需要前置条件的菜单项 */
  disabled?: boolean
}

const props = defineProps<{
  modelValue: string
  categories: NavCategory[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', id: string): void
}>()

/** 展开状态：按父项 id 记录 */
const expandedMap = ref<Record<string, boolean>>({})

/** 所有子项的 id → 父项 id 映射（用于根据 modelValue 自动展开父项） */
const childToParent = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const cat of props.categories) {
    if (cat.children) {
      for (const child of cat.children) {
        map[child.id] = cat.id
      }
    }
  }
  return map
})

/** 判断分类是否展开 */
function isExpanded(id: string): boolean {
  return !!expandedMap.value[id]
}

/** 切换展开状态 */
function toggleExpand(id: string) {
  expandedMap.value = { ...expandedMap.value, [id]: !expandedMap.value[id] }
}

/** 父项点击：disabled 不响应；有 children 则 toggle 展开，无 children 则 emit id */
function handleClick(cat: NavCategory) {
  if (cat.disabled) return
  if (cat.children && cat.children.length > 0) {
    toggleExpand(cat.id)
  } else {
    emit('update:modelValue', cat.id)
  }
}

/** 判断父项是否高亮（自身选中或子项选中） */
function isParentActive(cat: NavCategory): boolean {
  if (props.modelValue === cat.id) return true
  if (cat.children) {
    return cat.children.some(c => c.id === props.modelValue)
  }
  return false
}

// modelValue 变化时：如果是某父项的子项，自动展开该父项
watch(() => props.modelValue, (val) => {
  const parentId = childToParent.value[val]
  if (parentId && !isExpanded(parentId)) {
    expandedMap.value = { ...expandedMap.value, [parentId]: true }
  }
}, { immediate: true })

// tab 选中态 URL 持久化（onMounted 恢复 + watch 写入，逻辑抽取到 useTabPersistence）
useTabPersistence(
  () => props.modelValue,
  (tab) => {
    // 校验 tab 是否在 categories 中（含 children）且未禁用
    for (const cat of props.categories) {
      if (cat.id === tab && !cat.disabled) return true
      if (cat.children) {
        for (const child of cat.children) {
          if (child.id === tab && !child.disabled) return true
        }
      }
    }
    return false
  },
  (tab) => emit('update:modelValue', tab),
)
</script>

<template>
  <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
    <div class="flex-1 overflow-y-auto py-4">
      <div v-for="cat in categories" :key="cat.id">
        <!-- 父项（无 children 时为普通项，有 children 时为可展开项） -->
        <button
          type="button"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            cat.disabled
              ? 'text-gray-300 cursor-not-allowed'
              : isParentActive(cat)
                ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500 cursor-pointer'
                : 'text-gray-700 hover:bg-gray-50 cursor-pointer',
          ]"
          @click="handleClick(cat)"
        >
          <component :is="cat.icon" class="w-5 h-5 mr-3 shrink-0" />
          <span class="flex-1 text-left">{{ cat.label }}</span>
          <!-- 展开图标（仅有 children 时显示，带旋转动画） -->
          <ChevronDownIcon
            v-if="cat.children && cat.children.length > 0"
            class="w-4 h-4 text-gray-400 transition-transform duration-200"
            :class="isExpanded(cat.id) ? 'rotate-180' : ''"
          />
        </button>

        <!-- 子菜单（grid-template-rows 动画：0fr → 1fr） -->
        <div
          v-if="cat.children && cat.children.length > 0"
          class="grid transition-all duration-200 ease-out"
          :class="isExpanded(cat.id) ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'"
        >
          <div class="overflow-hidden">
            <button
              v-for="child in cat.children"
              :key="child.id"
              type="button"
              class="w-full flex items-center pl-11 pr-4 py-2 text-sm transition-colors"
              :class="[
                child.disabled
                  ? 'text-gray-300 cursor-not-allowed'
                  : modelValue === child.id
                    ? 'text-primary-700 bg-primary-50/50 font-medium cursor-pointer'
                    : 'text-gray-600 hover:bg-gray-50 cursor-pointer',
              ]"
              @click="!child.disabled && emit('update:modelValue', child.id)"
            >
              <component :is="child.icon" class="w-4 h-4 mr-2.5 shrink-0" />
              {{ child.label }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>
