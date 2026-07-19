<script setup lang="ts">
/**
 * 下载页左侧分类菜单（从 Versions.vue 抽出）
 *
 * 包含：
 * - 官方下载分类（原版/模组加载器/整合包）
 * - 社区资源分类（Mod/整合包/资源包/光影/数据包）
 * - 底部"打开游戏目录"按钮
 */
import { FolderOpenIcon } from '@heroicons/vue/24/outline'
import type { ResourceType } from '@/types/community'

interface Category {
  id: string
  label: string
  icon: any
}

interface CommunityCategory extends Category {
  type: ResourceType
}

defineProps<{
  /** 顶部官方下载分类 */
  topCategories: Category[]
  /** 社区资源分类 */
  communityCategories: CommunityCategory[]
  /** 当前选中的分类 ID */
  activeCategory: string
}>()

defineEmits<{
  /** 点击分类项（父组件需同步清空 selectedVersion） */
  select: [category: string]
  /** 点击"打开游戏目录" */
  openGameDir: []
}>()
</script>

<template>
  <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
    <div class="flex-1 overflow-y-auto py-4">
      <!-- 官方下载 -->
      <button
        v-for="cat in topCategories"
        :key="cat.id"
        class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
        :class="activeCategory === cat.id
          ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
          : 'text-gray-700 hover:bg-gray-50'"
        @click="$emit('select', cat.id)"
      >
        <component :is="cat.icon" class="w-5 h-5 mr-3" />
        {{ cat.label }}
      </button>

      <!-- 分隔线 -->
      <div class="my-2 mx-4 border-t border-gray-200"></div>

      <!-- 社区资源分组标题 -->
      <div class="px-4 py-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wide">社区资源</div>

      <!-- 社区子分类 -->
      <button
        v-for="cat in communityCategories"
        :key="cat.id"
        class="w-full flex items-center pl-8 pr-4 py-2 text-sm transition-colors"
        :class="activeCategory === cat.id
          ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
          : 'text-gray-600 hover:bg-gray-50'"
        @click="$emit('select', cat.id)"
      >
        <component :is="cat.icon" class="w-4 h-4 mr-2.5" />
        {{ cat.label }}
      </button>
    </div>
    <div class="p-3 border-t border-gray-200">
      <button
        class="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
        @click="$emit('openGameDir')"
      >
        <FolderOpenIcon class="w-4 h-4 mr-2" />
        打开游戏目录
      </button>
    </div>
  </aside>
</template>
