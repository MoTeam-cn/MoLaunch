<script setup lang="ts">
/**
 * 设置 - 开发者页面（薄编排层）
 *
 * 顶部子菜单分为：实验性功能 / 证书与安全 / 日志 / 存储 / 系统信息，
 * 五个子页签已拆分到 ./developer/ 目录：
 * - 实验性功能：Modrinth CDN 直连 → ExperimentalTab
 * - 证书与安全：TLS 信任源 + 忽略 TLS + 自定义证书管理 → CertsTab
 * - 日志：HTTP 请求日志 + 应用日志 → LogsTab
 * - 存储：缓存目录 + 存储信息 → StorageTab
 * - 系统信息：应用版本 / OS / 内存等 → SystemTab
 *
 * 数据来源：storageDirs / systemInfo 由本文件统一加载并经 props 下发；
 * ExperimentalTab / CertsTab 各自加载所需配置，保持职责内聚。
 */
import { ref, onMounted } from 'vue'
import SubTabBar from '@/components/common/SubTabBar.vue'
import * as tauri from '@/utils/tauri'
import { toastError } from '@/utils/toast'
import {
  BeakerIcon,
  ShieldCheckIcon,
  DocumentTextIcon,
  FolderOpenIcon,
  CpuChipIcon,
} from '@heroicons/vue/24/outline'
import ExperimentalTab from './developer/ExperimentalTab.vue'
import CertsTab from './developer/CertsTab.vue'
import LogsTab from './developer/LogsTab.vue'
import StorageTab from './developer/StorageTab.vue'
import SystemTab from './developer/SystemTab.vue'

// ── 子页签 ──
const subTabs = [
  { id: 'experimental', label: '实验性功能', icon: BeakerIcon },
  { id: 'certs', label: '证书与安全', icon: ShieldCheckIcon },
  { id: 'logs', label: '日志', icon: DocumentTextIcon },
  { id: 'storage', label: '存储', icon: FolderOpenIcon },
  { id: 'system', label: '系统信息', icon: CpuChipIcon },
]
const activeSubTab = ref('experimental')

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
      <CertsTab v-else-if="activeSubTab === 'certs'" />
      <LogsTab v-else-if="activeSubTab === 'logs'" :logs-dir="storageDirs?.logs" />
      <StorageTab v-else-if="activeSubTab === 'storage'" :storage-dirs="storageDirs" />
      <SystemTab v-else-if="activeSubTab === 'system'" :system-info="systemInfo" />
    </div>
  </div>
</template>
