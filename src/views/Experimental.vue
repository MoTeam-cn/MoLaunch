<script setup lang="ts">
/**
 * 实验性页面
 *
 * 需在「设置 → 进阶设置」开启实验性功能后，顶部导航才会显示本页入口。
 * 子分类：AI 聊天（Agent 对话，SQLite 存储会话）/ 日志分析 / AI 设置。
 * 后续将在此页面持续扩展更多实验性能力。
 */
import { ref, computed } from 'vue'
import {
  ChatBubbleLeftRightIcon,
  BugAntIcon,
  CogIcon,
  ShieldCheckIcon,
} from '@heroicons/vue/24/outline'
import NavSidebar from '@/components/common/NavSidebar.vue'
import ExperimentalChat from './experimental/ExperimentalChat.vue'
import ExperimentalLog from './experimental/ExperimentalLog.vue'
import SettingsAi from './settings/SettingsAi.vue'
import { useExperimental } from '@/composables/useExperimental'

const categories = [
  { id: 'chat', label: 'AI 聊天', icon: ChatBubbleLeftRightIcon, desc: '与 AI 对话，支持工具自动分析日志与崩溃' },
  { id: 'log', label: '日志分析', icon: BugAntIcon, desc: '规则引擎崩溃日志分析（仅本页可用）' },
  { id: 'ai-settings', label: 'AI 设置', icon: CogIcon, desc: '本地 OpenAI 兼容服务配置' },
]

const activeCategory = ref('chat')
const { enabled } = useExperimental()
const guarded = computed(() => enabled.value)
const activeDesc = () => categories.find((c) => c.id === activeCategory.value)?.desc ?? ''
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <NavSidebar v-model="activeCategory" :categories="categories" />

    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <div class="flex items-center gap-2">
          <ShieldCheckIcon class="w-5 h-5 text-primary-500" />
          <h2 class="text-lg font-semibold text-gray-900">
            {{ categories.find((c) => c.id === activeCategory)?.label }}
          </h2>
        </div>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc() }}</p>
      </div>

      <div v-if="!guarded" class="flex-1 flex flex-col items-center justify-center text-gray-400 p-6">
        <ShieldCheckIcon class="w-10 h-10 mb-2" />
        <span class="text-sm">实验性功能未开启</span>
        <p class="text-xs mt-1 text-gray-300">请在「设置 → 进阶设置」中启用实验性功能后使用</p>
      </div>

      <div v-else class="flex-1 overflow-hidden">
        <ExperimentalChat v-if="activeCategory === 'chat'" />
        <div v-else-if="activeCategory === 'log'" class="h-full overflow-y-auto p-6">
          <ExperimentalLog />
        </div>
        <div v-else-if="activeCategory === 'ai-settings'" class="h-full overflow-y-auto p-6">
          <SettingsAi />
        </div>
      </div>
    </div>
  </div>
</template>