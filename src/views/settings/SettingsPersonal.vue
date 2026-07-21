<script setup lang="ts">
/**
 * 设置 - 个性化页面
 *
 * - 外观：主题色、启动器语言
 * - 游戏：默认界面语言（写入 options.txt 的 lang 字段）
 *
 * 主题色/启动器语言存储在前端 localStorage（settingsStore），
 * 游戏默认语言存储在后端 INI（通过 applyConfig / getConfigMap IPC）。
 */
import { ref, watch, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { getConfigMap, applyConfig } from '@/utils/api/config'
import Select from '@/components/common/Select.vue'
import ColorPicker from '@/components/common/ColorPicker.vue'

const settingsStore = useSettingsStore()

// 游戏默认语言（后端配置，写入 options.txt 的 lang 字段）
// "none"=不设置 / "zh_cn" / "en_us" / "ja_jp" / "ko_kr" 等
// 默认 zh_cn（启动器语言固定简体中文，无需"跟随启动器"选项）
const gameLanguage = ref('zh_cn')

async function loadGameLanguage() {
  try {
    const cfg = await getConfigMap()
    // 兼容旧配置：若读到 auto，回退为 zh_cn
    const val = cfg.gameLanguage
    gameLanguage.value = !val || val === 'auto' ? 'zh_cn' : val
  } catch (e) {
    console.error('Failed to load game language:', e)
  }
}

async function saveGameLanguage(value: string | number) {
  try {
    await applyConfig({ gameLanguage: String(value) })
  } catch (e) {
    console.error('Failed to save game language:', e)
  }
}

watch(gameLanguage, (newLang) => {
  saveGameLanguage(newLang)
})

onMounted(loadGameLanguage)
</script>

<template>
  <div class="space-y-6">
    <!-- 外观 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外观</h3>
      <div class="divide-y divide-gray-200">
        <!-- 主题色（Arco Design 风格颜色选择器，控制所有 primary 蓝色区域） -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">主题色</p>
              <p class="text-xs text-gray-500 mt-0.5">控制菜单栏、按钮、选中态等所有主色区域</p>
            </div>
            <div class="flex-none w-40">
              <ColorPicker
                :model-value="settingsStore.primaryColor"
                @update:model-value="settingsStore.setPrimaryColor($event)"
              />
            </div>
          </div>
        </div>
        <!-- 启动器语言 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">启动器语言</p>
              <p class="text-xs text-gray-500 mt-0.5">启动器界面语言，暂时仅支持简体中文</p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="settingsStore.language"
                :options="[
                  { label: '简体中文', value: 'zh-CN' },
                ]"
                @update:model-value="settingsStore.setLanguage(String($event))"
              />
            </div>
          </div>
        </div>
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
