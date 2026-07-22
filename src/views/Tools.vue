<script setup lang="ts">
/**
 * 工具页
 *
 * 采用与 Settings.vue 一致的侧边栏布局：
 * - 左侧 w-48 菜单（activeCategory 切换高亮）
 * - 右侧内容区 v-if 切换子组件
 *
 * 分类：外部下载 / 便捷工具 / 存档管理 / Mod 工具 / 网络工具 / 计算工具 / 数据工具
 */

import { ref } from 'vue'
import {
  ArrowDownTrayIcon,
  WrenchScrewdriverIcon,
  ArchiveBoxIcon,
  PuzzlePieceIcon,
  SignalIcon,
  CalculatorIcon,
  CircleStackIcon,
} from '@heroicons/vue/24/outline'
import ExternalDownload from './ExternalDownload.vue'
import QuickTools from './QuickTools.vue'
import ArchivePage from './tools/archive/ArchivePage.vue'
import ModToolsPage from './tools/mod-tools/ModToolsPage.vue'
import NetworkPage from './tools/network/NetworkPage.vue'
import CalcPage from './tools/calc/CalcPage.vue'
import DataPage from './tools/data/DataPage.vue'

interface ToolCategory {
  id: string
  label: string
  icon: typeof ArrowDownTrayIcon
  desc: string
}

const categories: ToolCategory[] = [
  {
    id: 'external-download',
    label: '外部下载',
    icon: ArrowDownTrayIcon,
    desc: '通过 URL 下载任意文件，支持自定义目录、暂停、取消和进度展示',
  },
  {
    id: 'quick-tools',
    label: '便捷工具',
    icon: WrenchScrewdriverIcon,
    desc: '清理游戏垃圾、内存优化等实用工具',
  },
  {
    id: 'archive',
    label: '存档管理',
    icon: ArchiveBoxIcon,
    desc: '备份、恢复和导出游戏世界存档',
  },
  {
    id: 'mod-tools',
    label: 'Mod 工具',
    icon: PuzzlePieceIcon,
    desc: 'Mod 依赖检测、文件去重等 Mod 管理辅助工具',
  },
  {
    id: 'network',
    label: '网络工具',
    icon: SignalIcon,
    desc: '服务器状态检测、网络延迟测试',
  },
  {
    id: 'calc',
    label: '计算工具',
    icon: CalculatorIcon,
    desc: '坐标距离计算、游戏内调色板等计算辅助工具',
  },
  {
    id: 'data',
    label: '数据工具',
    icon: CircleStackIcon,
    desc: 'Java 管理、崩溃分析、JSON 编辑、NBT 查看、截图管理、数据导出、资源包转换',
  },
]

const activeCategory = ref<string>('quick-tools')

const activeDesc = () =>
  categories.find((c) => c.id === activeCategory.value)?.desc ?? ''
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单 -->
    <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            activeCategory === cat.id
              ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
              : 'text-gray-700 hover:bg-gray-50',
          ]"
          @click="activeCategory = cat.id"
        >
          <component :is="cat.icon" class="w-5 h-5 mr-3" />
          {{ cat.label }}
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ categories.find((c) => c.id === activeCategory)?.label }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc() }}</p>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <ExternalDownload v-if="activeCategory === 'external-download'" />
        <QuickTools v-else-if="activeCategory === 'quick-tools'" />
        <ArchivePage v-else-if="activeCategory === 'archive'" />
        <ModToolsPage v-else-if="activeCategory === 'mod-tools'" />
        <NetworkPage v-else-if="activeCategory === 'network'" />
        <CalcPage v-else-if="activeCategory === 'calc'" />
        <DataPage v-else-if="activeCategory === 'data'" />
      </div>
    </div>
  </div>
</template>
