<script setup lang="ts">
/**
 * 设置 - 开发者页面（薄编排层）
 *
 * 顶部子菜单分为：实验性功能 / DevTools / 证书与安全 / 日志 / 存储 / 系统信息 / 深链接，
 * 七个子页签已拆分到 ./developer/ 目录：
 * - 实验性功能：Modrinth CDN 直连 → ExperimentalTab
 * - DevTools：WebView2 开发者工具调出/关闭 + 测试版水印隐藏解锁 → DevToolsTab
 * - 证书与安全：TLS 信任源 + 忽略 TLS + 自定义证书管理 → CertsTab
 * - 日志：HTTP 请求日志 + 应用日志 → LogsTab
 * - 存储：缓存目录 + 存储信息 → StorageTab
 * - 系统信息：应用版本 / OS / 内存等 → SystemTab
 * - 深链接：molaunch:// 协议注册状态查询/注册/卸载（便携版用）→ DeepLinkTab
 *
 * 数据来源：storageDirs / systemInfo 由本文件统一加载并经 props 下发；
 * ExperimentalTab / CertsTab 各自加载所需配置，保持职责内聚。
 *
 * 开发者页面独占快捷键：
 * - Ctrl/Cmd + Shift + D：切换 DevTools 打开/关闭
 * - Alt + 1~7：切换子页签（1=实验性 / 2=DevTools / 3=证书 / 4=日志 / 5=存储 / 6=系统信息 / 7=深链接）
 * - 仅在本组件存活时生效（onUnmounted 自动解绑），由 useDevShortcuts 在 capture
 *   阶段 stopImmediatePropagation 抢占事件流，绕过 useDevToolsGuard 全局防护
 */
import { ref, onMounted } from 'vue'
import SubTabBar from '@/components/common/SubTabBar.vue'
import * as tauri from '@/utils/tauri'
import { toastError } from '@/utils/toast'
import { useDevShortcuts } from '@/composables/useDevShortcuts'
import {
  BeakerIcon,
  CommandLineIcon,
  ShieldCheckIcon,
  DocumentTextIcon,
  FolderOpenIcon,
  CpuChipIcon,
  LinkIcon,
} from '@heroicons/vue/24/outline'
import ExperimentalTab from './developer/ExperimentalTab.vue'
import DevToolsTab from './developer/DevToolsTab.vue'
import CertsTab from './developer/CertsTab.vue'
import LogsTab from './developer/LogsTab.vue'
import StorageTab from './developer/StorageTab.vue'
import SystemTab from './developer/SystemTab.vue'
import DeepLinkTab from './developer/DeepLinkTab.vue'

// ── 子页签 ──
const subTabs = [
  { id: 'experimental', label: '实验性功能', icon: BeakerIcon },
  { id: 'devtools', label: 'DevTools', icon: CommandLineIcon },
  { id: 'certs', label: '证书与安全', icon: ShieldCheckIcon },
  { id: 'logs', label: '日志', icon: DocumentTextIcon },
  { id: 'storage', label: '存储', icon: FolderOpenIcon },
  { id: 'system', label: '系统信息', icon: CpuChipIcon },
  { id: 'deeplink', label: '深链接', icon: LinkIcon },
]
const activeSubTab = ref('experimental')

// ── 开发者页面独占快捷键 ──
// Alt+1~6 切换子页签；Ctrl+Shift+D 切换 DevTools（由 composable 内部处理）
useDevShortcuts({
  onSwitchTab: (index: number) => {
    const tab = subTabs[index]
    if (tab) activeSubTab.value = tab.id
  },
})

// ── 共享数据（storageDirs / systemInfo 统一加载，props 下发） ──
const storageDirs = ref<tauri.StorageDirs | null>(null)
const systemInfo = ref<tauri.SystemInfo | null>(null)

async function loadStorageDirs() {
  try {
    storageDirs.value = await tauri.getStorageDirs()
  } catch (e) {
    console.error('Failed to load storage dirs:', e)
    toastError('获取存储目录失败：' + e)
  }
}

async function loadSystemInfo() {
  try {
    systemInfo.value = await tauri.getSystemInfo()
  } catch (e) {
    console.error('Failed to load system info:', e)
    toastError('获取系统信息失败：' + e)
  }
}

onMounted(async () => {
  await Promise.all([
    loadStorageDirs(),
    loadSystemInfo(),
  ])
})
</script>

<template>
  <div>
    <!-- 顶部子菜单（sticky 固定，滚动时吸顶紧贴标题栏） -->
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <!-- 内容区 -->
    <div class="space-y-6 p-6">
      <ExperimentalTab v-if="activeSubTab === 'experimental'" />
      <DevToolsTab v-else-if="activeSubTab === 'devtools'" />
      <CertsTab v-else-if="activeSubTab === 'certs'" />
      <LogsTab v-else-if="activeSubTab === 'logs'" :logs-dir="storageDirs?.logs" />
      <StorageTab v-else-if="activeSubTab === 'storage'" :storage-dirs="storageDirs" />
      <SystemTab v-else-if="activeSubTab === 'system'" :system-info="systemInfo" />
      <DeepLinkTab v-else-if="activeSubTab === 'deeplink'" />
    </div>
  </div>
</template>
