<script setup lang="ts">
/**
 * 通用侧边栏组件
 * 支持二级、三级菜单
 */

import { ref, computed } from 'vue'

export interface SidebarItem {
  id: string
  label: string
  icon?: string
  description?: string
  children?: SidebarItem[]
  badge?: string | number
  disabled?: boolean
}

interface Props {
  items: SidebarItem[]
  activeId?: string
  title?: string
  description?: string
}

const props = withDefaults(defineProps<Props>(), {
  activeId: '',
  title: '',
  description: '',
})

const emit = defineEmits<{
  select: [id: string]
}>()

// 展开的项目
const expandedItems = ref<Set<string>>(new Set())

function toggleExpand(id: string) {
  if (expandedItems.value.has(id)) {
    expandedItems.value.delete(id)
  } else {
    expandedItems.value.add(id)
  }
}

function isExpanded(id: string): boolean {
  return expandedItems.value.has(id)
}

function handleSelect(id: string) {
  emit('select', id)
}

function hasChildren(item: SidebarItem): boolean {
  return item.children && item.children.length > 0
}
</script>

<template>
  <div class="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col h-full">
    <!-- 标题区域 -->
    <div v-if="title || description" class="p-4 border-b border-gray-200 dark:border-gray-700">
      <h3 v-if="title" class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ title }}
      </h3>
      <p v-if="description" class="text-xs text-gray-500 dark:text-gray-400 mt-1">
        {{ description }}
      </p>
    </div>

    <!-- 菜单列表 -->
    <div class="flex-1 overflow-y-auto py-2">
      <div v-for="item in items" :key="item.id" class="px-2">
        <!-- 一级菜单 -->
        <div
          class="flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer transition-colors"
          :class="[
            activeId === item.id
              ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
              : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700',
            item.disabled ? 'opacity-50 cursor-not-allowed' : ''
          ]"
          @click="!item.disabled && (hasChildren(item) ? toggleExpand(item.id) : handleSelect(item.id))"
        >
          <div class="flex items-center min-w-0">
            <!-- 图标 -->
            <span v-if="item.icon" class="mr-2 text-lg">{{ item.icon }}</span>
            
            <!-- 标签和描述 -->
            <div class="min-w-0">
              <div class="flex items-center">
                <span class="text-sm font-medium truncate">{{ item.label }}</span>
                <span
                  v-if="item.badge !== undefined"
                  class="ml-2 text-xs px-1.5 py-0.5 rounded-full bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300"
                >
                  {{ item.badge }}
                </span>
              </div>
              <p v-if="item.description" class="text-xs text-gray-500 dark:text-gray-400 truncate">
                {{ item.description }}
              </p>
            </div>
          </div>

          <!-- 展开箭头 -->
          <svg
            v-if="hasChildren(item)"
            class="w-4 h-4 transition-transform"
            :class="{ 'rotate-90': isExpanded(item.id) }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </div>

        <!-- 二级菜单 -->
        <transition
          enter-active-class="transition ease-out duration-100"
          enter-from-class="opacity-0 -translate-y-1"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition ease-in duration-75"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 -translate-y-1"
        >
          <div v-if="hasChildren(item) && isExpanded(item.id)" class="ml-4 mt-1">
            <div v-for="child in item.children" :key="child.id" class="px-2">
              <!-- 二级菜单项 -->
              <div
                class="flex items-center justify-between px-3 py-1.5 rounded-lg cursor-pointer transition-colors"
                :class="[
                  activeId === child.id
                    ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                    : 'text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-700/50',
                  child.disabled ? 'opacity-50 cursor-not-allowed' : ''
                ]"
                @click="!child.disabled && (hasChildren(child) ? toggleExpand(child.id) : handleSelect(child.id))"
              >
                <div class="flex items-center min-w-0">
                  <span v-if="child.icon" class="mr-2 text-sm">{{ child.icon }}</span>
                  <div class="min-w-0">
                    <span class="text-sm truncate">{{ child.label }}</span>
                    <p v-if="child.description" class="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {{ child.description }}
                    </p>
                  </div>
                </div>

                <div class="flex items-center">
                  <span
                    v-if="child.badge !== undefined"
                    class="text-xs px-1.5 py-0.5 rounded-full bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300"
                  >
                    {{ child.badge }}
                  </span>
                  <svg
                    v-if="hasChildren(child)"
                    class="w-3.5 h-3.5 ml-1 transition-transform"
                    :class="{ 'rotate-90': isExpanded(child.id) }"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </div>
              </div>

              <!-- 三级菜单 -->
              <transition
                enter-active-class="transition ease-out duration-100"
                enter-from-class="opacity-0 -translate-y-1"
                enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition ease-in duration-75"
                leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-1"
              >
                <div v-if="hasChildren(child) && isExpanded(child.id)" class="ml-4 mt-1">
                  <div
                    v-for="grandchild in child.children"
                    :key="grandchild.id"
                    class="px-3 py-1.5 rounded-lg cursor-pointer transition-colors"
                    :class="[
                      activeId === grandchild.id
                        ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                        : 'text-gray-500 hover:bg-gray-50 dark:text-gray-500 dark:hover:bg-gray-700/50',
                      grandchild.disabled ? 'opacity-50 cursor-not-allowed' : ''
                    ]"
                    @click="!grandchild.disabled && handleSelect(grandchild.id)"
                  >
                    <div class="flex items-center">
                      <span v-if="grandchild.icon" class="mr-2 text-xs">{{ grandchild.icon }}</span>
                      <span class="text-xs truncate">{{ grandchild.label }}</span>
                    </div>
                  </div>
                </div>
              </transition>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>
