<script setup lang="ts">
/**
 * 设置 - 个性化页面
 *
 * - 外观：主题色、启动器语言 → AppearanceSection
 * - 主页：主页右侧内容区模式选择 → HomePanelModeSection
 * - 游戏：默认界面语言（写入 options.txt 的 lang 字段）
 */
import { ref, watch } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
import { useSettingsStore } from '@/stores/settings'
import Select from '@/components/common/Select.vue'
import ToggleRow from '@/components/settings/ToggleRow.vue'
import AppearanceSection from './personal/AppearanceSection.vue'
import HomePanelModeSection from './personal/HomePanelModeSection.vue'

const settingsStore = useSettingsStore()

// 游戏默认语言（后端配置，写入 options.txt 的 lang 字段）
const gameLanguage = ref('zh_cn')

const { loaded: gameLanguageLoaded, markDirty: markGameLanguageDirty } = useConfigPage({
  delay: 500,
  errorLabel: 'save game language',
  onLoad: (cfg) => {
    const val = cfg.gameLanguage
    gameLanguage.value = !val || val === 'auto' ? 'zh_cn' : val
  },
})

watch(gameLanguage, (newLang) => {
  if (gameLanguageLoaded.value) markGameLanguageDirty('gameLanguage', String(newLang))
})

// 关闭主界面行为（ask 每次询问 / tray 保留托盘 / exit 直接退出）
const closeBehavior = ref('ask')

const { loaded: closeBehaviorLoaded, markDirty: markCloseBehaviorDirty } = useConfigPage({
  delay: 500,
  errorLabel: 'save close behavior',
  onLoad: (cfg) => {
    closeBehavior.value = cfg.closeBehavior || 'ask'
  },
})

watch(closeBehavior, (val) => {
  if (closeBehaviorLoaded.value) markCloseBehaviorDirty('closeBehavior', String(val))
})
</script>

<template>
  <div class="space-y-6">
    <!-- 外观 -->
    <AppearanceSection />

    <!-- 主页 -->
    <HomePanelModeSection />

    <!-- 主界面 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">主界面</h3>
      <div class="divide-y divide-gray-200">
        <!-- 关闭主界面时 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">关闭主界面时</p>
              <p class="text-xs text-gray-500 mt-0.5">选择"保留托盘"后关闭主界面仍会在系统托盘运行；可在托盘菜单中退出</p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="closeBehavior"
                :options="[
                  { label: '每次询问', value: 'ask' },
                  { label: '保留托盘', value: 'tray' },
                  { label: '直接退出', value: 'exit' },
                ]"
                @update:model-value="closeBehavior = String($event)"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 启动 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">启动</h3>
      <div class="divide-y divide-gray-200">
        <ToggleRow
          :model-value="settingsStore.rememberLastPage"
          label="记住上次打开的页面"
          description="启动时自动回到上次打开的页面（如设置页）；默认关闭，不记录浏览位置"
          :hover="false"
          @update:model-value="settingsStore.setRememberLastPage($event)"
        />
      </div>
    </div>

    <!-- 游戏 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">游戏</h3>
      <div class="divide-y divide-gray-200">
        <!-- 游戏默认界面语言 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">默认界面语言</p>
              <p class="text-xs text-gray-500 mt-0.5">启动游戏时自动设置 options.txt 的 lang 字段；已有存档的玩家语言不会被覆盖</p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="gameLanguage"
                :options="[
                  { label: '简体中文', value: 'zh_cn' },
                  { label: 'English', value: 'en_us' },
                  { label: '日本語', value: 'ja_jp' },
                  { label: '한국어', value: 'ko_kr' },
                  { label: 'Français', value: 'fr_fr' },
                  { label: 'Deutsch', value: 'de_de' },
                  { label: 'Русский', value: 'ru_ru' },
                  { label: '不设置', value: 'none' },
                ]"
                @update:model-value="gameLanguage = String($event)"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
