<script setup lang="ts">
/**
 * 设置 - 更多页面（薄编排层）
 *
 * 顶部子菜单分为：关于 / 鸣谢 / 教程，三个子页签已拆分到 ./more/ 目录：
 * - 关于：MoLaunch 介绍、官网链接、技术栈 → AboutTab
 * - 鸣谢：BMCLAPI / mcmod / MCIM API + 法律信息 + 许可与版权声明 → CreditsTab
 * - 教程：使用教程（占位）→ TutorialTab
 *
 * 数据来源：所有结构化数据通过后端 `get_about_data` IPC 命令加载，
 * 主文件统一管理加载状态并经 props 下发给子页签。
 */
import { ref, onMounted } from 'vue'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { getAboutData, type AboutData } from '@/utils/api/about'
import {
  InformationCircleIcon,
  HeartIcon,
  BookOpenIcon,
} from '@heroicons/vue/24/outline'
import AboutTab from './more/AboutTab.vue'
import CreditsTab from './more/CreditsTab.vue'
import TutorialTab from './more/TutorialTab.vue'

// ── 子页签 ──
const subTabs = [
  { id: 'about', label: '关于', icon: InformationCircleIcon },
  { id: 'credits', label: '鸣谢', icon: HeartIcon },
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

onMounted(loadAboutData)
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
      <CreditsTab
        v-else-if="activeSubTab === 'credits'"
        :about-data="aboutData"
        :loading="loading"
        :load-error="loadError"
      />
      <TutorialTab v-else-if="activeSubTab === 'tutorial'" />
    </div>
  </div>
</template>
