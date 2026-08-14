<script setup lang="ts">
/**
 * 首页（左右双栏布局）
 * 左侧：账号选择 + 版本选择 + 启动按钮
 * 右侧：根据插件设置渲染时钟卡片 / 启动日志 / 插件组件 / 自定义布局
 */

import { computed, ref, watch, onMounted, defineAsyncComponent } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useVersionStore } from '@/stores/version'
import { useJavaStore } from '@/stores/java'
import { usePluginStore } from '@/stores/plugins'
import { useVersionSettings } from '@/composables/useVersionSettings'
import * as tauri from '@/utils/tauri'
const LaunchPanel = defineAsyncComponent(() => import('@/components/home/LaunchPanel.vue'))
const LaunchLog = defineAsyncComponent(() => import('@/components/home/LaunchLog.vue'))
const HomeClockCard = defineAsyncComponent(() => import('@/components/home/HomeClockCard.vue'))
const CustomLayout = defineAsyncComponent(() => import('@/plugins/custom-layout/index.vue'))

const authStore = useAuthStore()
const versionStore = useVersionStore()
const javaStore = useJavaStore()
const pluginStore = usePluginStore()
const { refreshInstalledVersionTypes } = useVersionSettings()

/**
 * 是否显示启动进度（含 600ms 延迟隐藏，让用户看到 100%）
 *
 * 启动中（或刚结束的 600ms 内）渲染 LaunchLog，其余时间根据 homePanelMode 渲染。
 */
const showLaunchProgress = ref(false)
let hideTimer: number | null = null

watch(
  () => versionStore.launching,
  (launching) => {
    if (launching) {
      if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
      showLaunchProgress.value = true
    } else {
      hideTimer = window.setTimeout(() => {
        showLaunchProgress.value = false
        hideTimer = null
      }, 600)
    }
  },
  { immediate: true },
)

/**
 * 主页右侧内容区当前渲染的组件
 *
 * - 启动中：渲染 LaunchLog（进度条）
 * - homePanelMode = "default"：渲染 HomeClockCard（时钟卡片 + 自动翻页）
 * - homePanelMode = "plugin:<id>" 且插件已启用：渲染插件提供的 homePanel 组件
 * - homePanelMode = "custom"：渲染自定义布局组件（CustomLayout）
 *
 * 所有失败/回退情况统一渲染 HomeClockCard。
 */
const homePanelComponent = computed(() => {
  // 启动中（含 600ms 延迟）：渲染 LaunchLog
  if (showLaunchProgress.value) return LaunchLog

  const mode = pluginStore.homePanelMode

  // 自定义模式
  if (mode === 'custom') {
    return CustomLayout
  }

  // 插件模式
  if (mode.startsWith('plugin:')) {
    const pluginId = mode.slice('plugin:'.length)
    const manifest = pluginStore.manifests.find((m) => m.id === pluginId)
    if (!manifest) return HomeClockCard
    if (!pluginStore.runtimeStates[manifest.id]?.enabled) return HomeClockCard
    return manifest.capabilities?.()?.homePanel ?? HomeClockCard
  }

  // 默认模式：时钟卡片
  return HomeClockCard
})

/** 传递给右侧面板组件的 props
 *
 * 仅 custom 模式需要传 config，其他模式传空对象。
 * v-bind="{}" 等价于无 props，不影响 LaunchLog / 时钟卡片 / 插件组件渲染。
 */
const homePanelProps = computed(() => {
  if (pluginStore.homePanelMode === 'custom') {
    return { config: pluginStore.customLayoutConfig }
  }
  return {}
})

onMounted(async () => {
  // 阶段1：并行恢复会话 + 立即从后端读取上次选中的版本（仅一次 IPC 读 config，不校验，约 1ms）
  // 用户立即看到版本名 + 开始游戏按钮变蓝，无需等待磁盘扫描或网络请求
  // sdkStore.fetchPlatformInfo 已由 App.vue 触发，此处因 initialized guard 自动跳过
  await Promise.all([
    authStore.restoreSession(),
    javaStore.detectJava(),
    versionStore.restoreSelectedVersionFast(),
  ])

  // 阶段2：并行执行互不依赖的操作
  // - checkRunningGame：检测运行中游戏（IPC，快）
  // - listInstalledVersionsWithType：磁盘扫描已安装版本（中等耗时）
  // 注：fetchVersions（拉取 Mojang 版本清单 1~3s）已移除，首页不使用 versions 数组，
  //     仅版本选择页（VersionSelect.vue）在进入时按需拉取
  let installed: Awaited<ReturnType<typeof tauri.listInstalledVersionsWithType>> = []
  try {
    const [, scanned] = await Promise.all([
      versionStore.checkRunningGame(),
      tauri.listInstalledVersionsWithType().catch(() => [] as typeof installed),
    ])
    installed = scanned
  } catch {
    // 忽略：用户可去版本选择页手动选
  }

  // 阶段3：刷新已安装版本类型映射缓存 + 校验 selectedVersion 是否仍存在
  // （不存在则清空持久化并自动回退到第一个已安装版本）
  await refreshInstalledVersionTypes(installed)
  await versionStore.validateSelectedVersion(installed)
})
</script>

<template>
  <div class="flex h-full gap-2 p-2">
    <!-- 左侧启动面板（固定宽度 320px） -->
    <aside class="flex w-80 flex-none flex-col overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm">
      <LaunchPanel />
    </aside>

    <!-- 右侧内容区（弹性宽度，根据插件配置动态渲染） -->
    <main class="flex-1 overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm">
      <component :is="homePanelComponent" v-bind="homePanelProps" />
    </main>
  </div>
</template>
