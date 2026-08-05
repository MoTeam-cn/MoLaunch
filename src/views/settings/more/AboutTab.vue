<script setup lang="ts">
/**
 * 关于子页签：MoLaunch 介绍 + 实现原理 + 技术栈 + 检查更新入口
 */
import { computed, ref } from 'vue'
import Card from '@/components/common/Card.vue'
import Button from '@/components/common/Button.vue'
import Tag from '@/components/common/Tag.vue'
import MoLaunchIntro from '@/components/about/MoLaunchIntro.vue'
import { openLink } from '@/utils/aboutLogos'
import { checkForUpdate } from '@/utils/updater'
import type { AboutData } from '@/utils/api/about'
import logoMoLaunch from '@/assets/logo.svg'
import {
  GlobeAltIcon,
  CodeBracketIcon,
  ArrowTopRightOnSquareIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  aboutData: AboutData | null
  loading: boolean
  loadError: string
}>()

// 应用版本（vite define 注入的全局常量）
const appVersion = __APP_VERSION__

const frontendDeps = computed(() => props.aboutData?.frontendDeps ?? [])
const frontendDevDeps = computed(() => props.aboutData?.frontendDevDeps ?? [])
const backendDeps = computed(() => props.aboutData?.backendDeps ?? [])

// 检查更新按钮状态（仅按钮自身 loading，弹窗由 UpdateDialog 全局组件展示）
const checking = ref(false)

async function onCheckUpdate() {
  if (checking.value) return
  checking.value = true
  try {
    await checkForUpdate({ silent: false })
  } finally {
    checking.value = false
  }
}
</script>

<template>
  <!-- MoLaunch 介绍 -->
  <Card>
    <template #title>
      <div class="flex items-center gap-3">
        <img :src="logoMoLaunch" alt="MoLaunch" class="h-8 w-8" />
        <span class="text-base font-bold text-gray-900">MoLaunch</span>
        <Tag size="small" color="arcoblue">v{{ appVersion }}</Tag>
      </div>
    </template>
    <template #extra>
      <div class="flex items-center gap-2">
        <Button
          type="outline"
          size="small"
          :loading="checking"
          @click="onCheckUpdate"
        >
          <template #icon><ArrowPathIcon class="h-3.5 w-3.5" /></template>
          检查更新
        </Button>
        <Button type="text" size="small" @click="openLink('https://molaunch.moiu.cn')">
          <template #icon><GlobeAltIcon class="h-3.5 w-3.5" /></template>
          点我前往
        </Button>
      </div>
    </template>

    <div class="space-y-3">
      <p class="text-[13px] leading-relaxed text-gray-600">
        <span class="font-semibold text-gray-800">MoLaunch</span> 是一款面向所有 Minecraft 玩家的启动器，核心目标是消除联机障碍。
        内置 FRP 隧道模块，简化端口映射流程，让玩家专注游戏本身，而非网络配置。
      </p>
      <div class="grid grid-cols-3 gap-3">
        <div class="rounded-lg bg-gray-50 px-3 py-2">
          <div class="text-[11px] text-gray-400">项目愿景</div>
          <div class="mt-0.5 text-[12px] font-medium text-gray-700">让联机回归便捷</div>
        </div>
        <div class="rounded-lg bg-gray-50 px-3 py-2">
          <div class="text-[11px] text-gray-400">目标用户</div>
          <div class="mt-0.5 text-[12px] font-medium text-gray-700">所有 MC 玩家</div>
        </div>
        <div class="rounded-lg bg-gray-50 px-3 py-2">
          <div class="text-[11px] text-gray-400">核心价值</div>
          <div class="mt-0.5 text-[12px] font-medium text-gray-700">简化联机流程</div>
        </div>
      </div>
    </div>
  </Card>

  <!-- MoLaunch 实现原理（默认折叠，点击展开） -->
  <MoLaunchIntro />

  <!-- 技术栈 -->
  <Card>
    <template #title>
      <div class="flex items-center gap-2">
        <CodeBracketIcon class="h-4 w-4 text-gray-500" />
        <span class="text-sm font-semibold text-gray-800">技术栈</span>
      </div>
    </template>

    <!-- 加载状态 -->
    <div v-if="loading" class="py-8 text-center text-[12px] text-gray-400">加载中...</div>
    <!-- 加载失败 -->
    <div v-else-if="loadError" class="py-8 text-center text-[12px] text-red-500">
      加载失败：{{ loadError }}
    </div>
    <!-- 数据展示 -->
    <div v-else class="space-y-5">
      <div>
        <div class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-400">前端</div>
        <div class="grid grid-cols-2 gap-2">
          <!-- 保留原生 button：依赖列表项为链接卡片（justify-between + 右侧外链图标），
               Button.vue 的 svg margin 与居中布局不适合列表项布局，下同 -->
          <button
            v-for="dep in frontendDeps"
            :key="dep.name"
            class="flex items-center justify-between rounded-md border border-gray-100 px-3 py-2 text-left transition-colors hover:border-primary-200 hover:bg-primary-50/30"
            @click="openLink(dep.url)"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="text-[12px] font-medium text-gray-700">{{ dep.name }}</span>
                <span class="text-[10px] text-gray-400">{{ dep.version }}</span>
              </div>
              <div class="truncate text-[11px] text-gray-400">{{ dep.desc }}</div>
            </div>
            <ArrowTopRightOnSquareIcon class="ml-2 h-3 w-3 flex-none text-gray-300" />
          </button>
        </div>
      </div>

      <div>
        <div class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-400">前端开发工具</div>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="dep in frontendDevDeps"
            :key="dep.name"
            class="flex items-center justify-between rounded-md border border-gray-100 px-3 py-2 text-left transition-colors hover:border-primary-200 hover:bg-primary-50/30"
            @click="openLink(dep.url)"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="text-[12px] font-medium text-gray-700">{{ dep.name }}</span>
                <span class="text-[10px] text-gray-400">{{ dep.version }}</span>
              </div>
              <div class="truncate text-[11px] text-gray-400">{{ dep.desc }}</div>
            </div>
            <ArrowTopRightOnSquareIcon class="ml-2 h-3 w-3 flex-none text-gray-300" />
          </button>
        </div>
      </div>

      <div>
        <div class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-400">后端 (Rust)</div>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="dep in backendDeps"
            :key="dep.name"
            class="flex items-center justify-between rounded-md border border-gray-100 px-3 py-2 text-left transition-colors hover:border-primary-200 hover:bg-primary-50/30"
            @click="openLink(dep.url)"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="text-[12px] font-medium text-gray-700">{{ dep.name }}</span>
                <span class="text-[10px] text-gray-400">{{ dep.version }}</span>
              </div>
              <div class="truncate text-[11px] text-gray-400">{{ dep.desc }}</div>
            </div>
            <ArrowTopRightOnSquareIcon class="ml-2 h-3 w-3 flex-none text-gray-300" />
          </button>
        </div>
      </div>
    </div>
  </Card>
</template>
