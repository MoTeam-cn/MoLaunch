<script setup lang="ts">
/**
 * 工具页
 *
 * 采用与 Settings.vue 一致的侧边栏布局：
 * - 左侧 w-48 菜单（activeCategory 切换高亮）
 * - 右侧内容区 v-if 切换子组件
 *
 * 分类（7 个一级菜单，内部子工具用顶部 SubTabBar 切换）：
 * 下载管理 / 常用工具 / 数据迁移 / 存档资源 / Mod 网络 / Java 诊断 / 创作指令
 */

import { ref, defineAsyncComponent } from 'vue'
import {
  ArrowDownTrayIcon,
  WrenchScrewdriverIcon,
  InboxArrowDownIcon,
  ArchiveBoxIcon,
  PuzzlePieceIcon,
  CommandLineIcon,
  PaintBrushIcon,
} from '@heroicons/vue/24/outline'
const NavSidebar = defineAsyncComponent(() => import('@/components/common/NavSidebar.vue'))
const ExternalDownload = defineAsyncComponent(() => import('./ExternalDownload.vue'))
const CommonPage = defineAsyncComponent(() => import('./tools/CommonPage.vue'))
const LauncherImportPage = defineAsyncComponent(() => import('./tools/LauncherImportPage.vue'))
const StoragePage = defineAsyncComponent(() => import('./tools/StoragePage.vue'))
const ModNetworkPage = defineAsyncComponent(() => import('./tools/ModNetworkPage.vue'))
const JavaDiagPage = defineAsyncComponent(() => import('./tools/JavaDiagPage.vue'))
const CreateCmdPage = defineAsyncComponent(() => import('./tools/CreateCmdPage.vue'))
const DisclaimerDialog = defineAsyncComponent(() => import('@/components/common/DisclaimerDialog.vue'))
import { hasAgreedToday } from '@/utils/disclaimer'

interface ToolCategory {
  id: string
  label: string
  icon: typeof ArrowDownTrayIcon
  desc: string
}

const categories: ToolCategory[] = [
  {
    id: 'external-download',
    label: '下载管理',
    icon: ArrowDownTrayIcon,
    desc: '通过 URL 下载任意文件，支持自定义目录、暂停、取消和进度展示',
  },
  {
    id: 'common',
    label: '常用工具',
    icon: WrenchScrewdriverIcon,
    desc: '今日人品、清理游戏垃圾、内存优化、版本 JSON 编辑等日常实用小工具',
  },
  {
    id: 'launcher-import',
    label: '数据迁移',
    icon: InboxArrowDownIcon,
    desc: '从 PCL2 / HMCL / MultiMC / CurseForge 等启动器导入实例，支持复制或符号链接',
  },
  {
    id: 'storage',
    label: '存档资源',
    icon: ArchiveBoxIcon,
    desc: '游戏世界存档备份/恢复/导出，截图批量管理、资源包转换、种子地图与 NBT 数据编辑',
  },
  {
    id: 'mod-network',
    label: 'Mod 网络',
    icon: PuzzlePieceIcon,
    desc: 'Mod 依赖检测/文件去重，服务器状态检测与网络延迟测试',
  },
  {
    id: 'java-diag',
    label: 'Java 诊断',
    icon: CommandLineIcon,
    desc: 'Java 运行时下载、环境检测与启动排障工具',
  },
  {
    id: 'create-cmd',
    label: '创作指令',
    icon: PaintBrushIcon,
    desc: '渐变文字/合成配方等创作工具，及物品编辑、告示牌商店、召唤实体指令生成',
  },
]

const activeCategory = ref<string>('common')

/** 使用协议抽屉：当日未同意过工具协议时弹出（同意后存 localStorage，次日重新提醒） */
const disclaimerVisible = ref(!hasAgreedToday('tools'))

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

      <!-- 内容区（滚动条保持在最右侧） -->
      <div class="flex-1 relative overflow-hidden">
        <!-- 各分类页自带 SubTabBar + p-6（顶部子菜单需贴边）；仅直接在 Tools.vue 渲染的分类需要外层 p-6 -->
        <div
          class="h-full overflow-y-auto tools-scroll-container"
          :class="activeCategory === 'external-download' ? 'p-6' : ''"
        >
          <ExternalDownload v-if="activeCategory === 'external-download'" />
          <CommonPage v-else-if="activeCategory === 'common'" />
          <LauncherImportPage v-else-if="activeCategory === 'launcher-import'" />
          <StoragePage v-else-if="activeCategory === 'storage'" />
          <ModNetworkPage v-else-if="activeCategory === 'mod-network'" />
          <JavaDiagPage v-else-if="activeCategory === 'java-diag'" />
          <CreateCmdPage v-else />
        </div>
      </div>
    </div>

    <!-- 使用协议抽屉（当日未同意时展示；teleport 到 #app-content，位置不影响单根约束） -->
    <DisclaimerDialog v-model:visible="disclaimerVisible" kind="tools" />
  </div>
</template>
