<script setup lang="ts">
/**
 * 教程子页签
 *
 * 展示教程列表（按分类分组），点击「阅读」调用 openTutorialWindow
 * 在 picker 子窗口中用 marked.min.js 渲染 Markdown 内容。
 *
 * 教程内容存储在 src/tutorials/*.md，通过 Vite ?raw 导入。
 * 新增教程只需在 src/tutorials/index.ts 中注册。
 */
import { computed } from 'vue'
import {
  BookOpenIcon,
  ArrowRightIcon,
  CodeBracketIcon,
  RocketLaunchIcon,
} from '@heroicons/vue/24/outline'
import Card from '@/components/common/Card.vue'
import Button from '@/components/common/Button.vue'
import { TUTORIALS, type TutorialCategory } from '@/tutorials'
import { openTutorialWindow } from '@/utils/picker-window'

/** 分类图标映射 */
const categoryIcon: Record<TutorialCategory, typeof BookOpenIcon> = {
  基础: RocketLaunchIcon,
  'FRP 开发': CodeBracketIcon,
}

/** 按分类分组 */
const groupedTutorials = computed(() => {
  const groups: Record<TutorialCategory, typeof TUTORIALS> = {
    基础: [],
    'FRP 开发': [],
  }
  for (const t of TUTORIALS) {
    if (!groups[t.category]) groups[t.category] = []
    groups[t.category].push(t)
  }
  return Object.entries(groups).filter(([, list]) => list.length > 0)
})

/** 打开教程 */
async function openTutorial(title: string, content: string) {
  await openTutorialWindow({ title, content })
}
</script>

<template>
  <div class="space-y-5">
    <Card v-for="[category, list] in groupedTutorials" :key="category">
      <template #title>
        <div class="flex items-center gap-2">
          <component :is="categoryIcon[category as TutorialCategory]"
            class="h-4 w-4 text-gray-500" />
          <span class="text-sm font-semibold text-gray-800">{{ category }}</span>
        </div>
      </template>

      <div class="space-y-3">
        <div
          v-for="tutorial in list"
          :key="tutorial.id"
          class="flex items-start gap-3 rounded-lg border border-gray-100 p-3.5 transition-colors hover:border-gray-200"
        >
          <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary-50">
            <BookOpenIcon class="h-4 w-4 text-primary-500" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium text-gray-900">{{ tutorial.title }}</div>
            <p class="mt-0.5 text-xs leading-relaxed text-gray-500">{{ tutorial.description }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0"
            @click="openTutorial(tutorial.title, tutorial.content)"
          >
            阅读
            <template #icon><ArrowRightIcon class="h-3.5 w-3.5" /></template>
          </Button>
        </div>
      </div>
    </Card>
  </div>
</template>
