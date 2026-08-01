<script setup lang="ts">
/**
 * Online 顶部标题栏
 *
 * 从 Online.vue 抽离：当前激活分类标题 + 描述 + 状态徽章 + 教程按钮 + 联机设置按钮。
 * 数据全部由父组件通过 props 传入（activeLabel / activeDesc / badge / showHelp），
 * 按钮点击通过 emit('goTutorial') / emit('goSettings') 上抛，由父组件跳转路由。
 *
 * 「教程」按钮仅在 FRP 子菜单（providers / tunnels / auth / logs）激活时显示，
 * 点击跳转到设置 - 更多 - 教程子页，提供厂商与认证开发文档。
 *
 * 复用项目自定义 Button / Tooltip 组件，不使用原生 HTML。
 */
import { Cog6ToothIcon, BookOpenIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'

/** 状态徽章结构（与 useOnlineNav.badge 返回值同形） */
interface StatusBadge {
  text: string
  dotClass: string
  wrapClass: string
}

defineProps<{
  activeLabel: string
  activeDesc: string
  badge: StatusBadge
  /** 是否显示「教程帮助」按钮（FRP 子菜单激活时显示） */
  showHelp?: boolean
}>()

defineEmits<{
  (e: 'goSettings'): void
  (e: 'goTutorial'): void
}>()
</script>

<template>
  <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold text-gray-900">{{ activeLabel }}</h2>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc }}</p>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium"
          :class="badge.wrapClass"
        >
          <span class="w-1.5 h-1.5 rounded-full mr-1.5" :class="badge.dotClass" />
          {{ badge.text }}
        </span>
        <Tooltip v-if="showHelp" text="厂商与认证开发教程">
          <Button type="outline" size="small" @click="$emit('goTutorial')">
            <template #icon><BookOpenIcon class="w-4 h-4" /></template>
            教程
          </Button>
        </Tooltip>
        <Tooltip text="联机设置">
          <Button type="ghost" size="small" @click="$emit('goSettings')">
            <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>
  </div>
</template>
