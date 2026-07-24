<script setup lang="ts">
/**
 * 工具页
 *
 * 采用与 Settings.vue 一致的侧边栏布局：
 * - 左侧 w-48 菜单（activeCategory 切换高亮）
 * - 右侧内容区 v-if 切换子组件
 *
 * 分类（每分组 ≤ 3 个工具，重要/高频工具单独一栏）：
 * 外部下载 / 便捷工具 / 存档管理 / Mod 工具 / 网络工具 / 计算工具
 * / Java 管理 / 诊断工具 / 游戏资源 / 种子地图
 */

import { ref, nextTick, watch } from 'vue'
import {
  ArrowDownTrayIcon,
  WrenchScrewdriverIcon,
  ArchiveBoxIcon,
  PuzzlePieceIcon,
  SignalIcon,
  CalculatorIcon,
  CommandLineIcon,
  BugAntIcon,
  SwatchIcon,
  MapIcon,
} from '@heroicons/vue/24/outline'
import NavSidebar from '@/components/common/NavSidebar.vue'
import ExternalDownload from './ExternalDownload.vue'
import QuickTools from './QuickTools.vue'
import ArchivePage from './tools/archive/ArchivePage.vue'
import ModToolsPage from './tools/mod-tools/ModToolsPage.vue'
import NetworkPage from './tools/network/NetworkPage.vue'
import CalcPage from './tools/calc/CalcPage.vue'
import JavaPage from './tools/java/JavaPage.vue'
import DiagnosticPage from './tools/diagnostic/DiagnosticPage.vue'
import GameResourcePage from './tools/game-resource/GameResourcePage.vue'
import SeedMapPage from './tools/seedmap/SeedMapPage.vue'
import ToolToc from '@/components/common/ToolToc.vue'

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
    desc: '清理游戏垃圾、内存优化、启动器数据导出等实用工具',
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
    id: 'java',
    label: 'Java 管理',
    icon: CommandLineIcon,
    desc: '管理 Java 运行时版本，启动游戏的核心依赖',
  },
  {
    id: 'diagnostic',
    label: '诊断工具',
    icon: BugAntIcon,
    desc: '崩溃日志分析、版本 JSON 编辑、NBT 数据查看',
  },
  {
    id: 'game-resource',
    label: '游戏资源',
    icon: SwatchIcon,
    desc: '截图批量管理、资源包格式转换',
  },
  {
    id: 'seedmap',
    label: '种子地图',
    icon: MapIcon,
    desc: '输入种子加载 Minecraft 建筑位置地图，支持群系/结构查询',
  },
]

const activeCategory = ref<string>('quick-tools')
// 分类切换时递增，触发 ToolToc 重新扫描（含 NavSidebar 从 URL query.tab 恢复时）
const tocRefreshKey = ref(0)

watch(activeCategory, () => {
  nextTick(() => {
    tocRefreshKey.value++
  })
})

const activeDesc = () =>
  categories.find((c) => c.id === activeCategory.value)?.desc ?? ''
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单（公共组件，tab 同步到 URL query） -->
    <NavSidebar v-model="activeCategory" :categories="categories" />

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ categories.find((c) => c.id === activeCategory)?.label }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc() }}</p>
      </div>

      <!-- 内容区（滚动条保持在最右侧，TOC 悬浮不占布局） -->
      <div class="flex-1 relative overflow-hidden">
        <div class="h-full overflow-y-auto p-6 tools-scroll-container">
          <ExternalDownload v-if="activeCategory === 'external-download'" />
          <QuickTools v-else-if="activeCategory === 'quick-tools'" />
          <ArchivePage v-else-if="activeCategory === 'archive'" />
          <ModToolsPage v-else-if="activeCategory === 'mod-tools'" />
          <NetworkPage v-else-if="activeCategory === 'network'" />
          <CalcPage v-else-if="activeCategory === 'calc'" />
          <JavaPage v-else-if="activeCategory === 'java'" />
          <DiagnosticPage v-else-if="activeCategory === 'diagnostic'" />
          <GameResourcePage v-else-if="activeCategory === 'game-resource'" />
          <SeedMapPage v-else-if="activeCategory === 'seedmap'" />
        </div>
        <!-- 右侧悬浮 TOC 导航条（工具数 ≥ 3 时自动显示，不跟随滚动） -->
        <ToolToc :refresh-key="tocRefreshKey" :scroll-offset="20" />
      </div>
    </div>
  </div>
</template>
