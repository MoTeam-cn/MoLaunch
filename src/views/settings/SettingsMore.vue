<script setup lang="ts">
/**
 * 设置 - 更多页面（薄编排层）
 *
 * 顶部子菜单分为：关于 / 系统信息 / 鸣谢 / 许可协议 / 教程，子页签已拆分到 ./more/ 目录：
 * - 关于：MoLaunch 介绍、官网链接、技术栈 → AboutTab
 * - 系统信息：应用版本、开发者模式解锁、SDK 信息 → SystemInfoTab
 * - 鸣谢：BMCLAPI / mcmod / MCIM API + 法律信息 + 许可与版权声明 → CreditsTab
 * - 许可协议：项目 LICENSE 全文备份（build.rs 同步 + include_str! 嵌入二进制）→ LicenseTab
 * - 教程：启动器基础 + FRP 厂商开发指南（picker 子窗口渲染 Markdown）→ TutorialTab
 *
 * 数据来源：所有结构化数据通过后端 `get_about_data` IPC 命令加载，
 * 主文件统一管理加载状态并经 props 下发给子页签。
 *
 * 深链支持：URL `?subtab=tutorial` 可直接切到「教程」子页签（联机页 FRP 子菜单
 * 「教程」按钮通过 `/apps/settings?tab=about&subtab=tutorial` 跳转，
 * 由 SubTabBar 内置 URL 持久化恢复）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import { getAboutData, type AboutData } from '@/utils/api/about'
import {
  InformationCircleIcon,
  CpuChipIcon,
  HeartIcon,
  ScaleIcon,
  BookOpenIcon,
} from '@heroicons/vue/24/outline'
const AboutTab = defineAsyncComponent(() => import('./more/AboutTab.vue'))
const SystemInfoTab = defineAsyncComponent(() => import('./more/SystemInfoTab.vue'))
const CreditsTab = defineAsyncComponent(() => import('./more/CreditsTab.vue'))
const LicenseTab = defineAsyncComponent(() => import('./more/LicenseTab.vue'))
const TutorialTab = defineAsyncComponent(() => import('./more/TutorialTab.vue'))

// ── 子页签 ──
const subTabs = [
  { id: 'about', label: '关于', icon: InformationCircleIcon },
  { id: 'system', label: '系统信息', icon: CpuChipIcon },
  { id: 'credits', label: '鸣谢', icon: HeartIcon },
  { id: 'license', label: '许可协议', icon: ScaleIcon },
  { id: 'tutorial', label: '教程', icon: BookOpenIcon },
]
const activeSubTab = ref('about')

// ── 关于页面数据（异步加载，统一管理后下发给子页签） ──
const aboutData = ref<AboutData | null>(null)
const loading = ref(true)
const loadError = ref('')

async function loadAboutData() {
  loading.value = true
  loadError.value = ''
  try {
    aboutData.value = await getAboutData()
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : String(err)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadAboutData()
})
</script>

<template>
  <div>
    <!-- 顶部子菜单（sticky 固定，滚动时吸顶紧贴标题栏） -->
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <!-- 内容区 -->
    <div class="space-y-6 p-6">
      <AboutTab
        v-if="activeSubTab === 'about'"
        :about-data="aboutData"
        :loading="loading"
        :load-error="loadError"
      />
      <SystemInfoTab v-else-if="activeSubTab === 'system'" />
      <CreditsTab
        v-else-if="activeSubTab === 'credits'"
        :about-data="aboutData"
        :loading="loading"
        :load-error="loadError"
      />
      <LicenseTab v-else-if="activeSubTab === 'license'" />
      <TutorialTab v-else-if="activeSubTab === 'tutorial'" />
    </div>
  </div>
</template>
