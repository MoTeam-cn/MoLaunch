<script setup lang="ts">
/**
 * 主页右侧内容区配置：模式选择 + 插件选择 + 自定义布局（CustomLayoutSection）
 */
import { computed, defineAsyncComponent } from 'vue'
import { usePluginStore } from '@/stores/plugins'
import { toastInfo } from '@/utils/toast'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const CustomLayoutSection = defineAsyncComponent(() => import('./CustomLayoutSection.vue'))

const pluginStore = usePluginStore()

/** 主页右侧内容区顶层模式：default / plugin / custom */
const panelMode = computed<'default' | 'plugin' | 'custom'>(() => {
  const mode = pluginStore.homePanelMode
  if (mode === 'custom') return 'custom'
  if (mode.startsWith('plugin:')) return 'plugin'
  return 'default'
})

/** 主页右侧内容区模式选项 */
const modeOptions = [
  { label: '默认（启动日志）', value: 'default' },
  { label: '插件模式', value: 'plugin' },
  { label: '自定义模式', value: 'custom' },
]

/** 插件模式下可选的插件列表 */
const pluginOptions = computed(() => {
  const opts: { label: string; value: string }[] = []
  for (const manifest of pluginStore.manifests) {
    const caps = manifest.capabilities?.()
    const enabled = pluginStore.runtimeStates[manifest.id]?.enabled
    if (caps?.homePanel && enabled) {
      opts.push({ label: manifest.name, value: `plugin:${manifest.id}` })
    }
  }
  return opts
})

/** 当前选中的插件值 */
const selectedPlugin = computed(() => {
  if (panelMode.value !== 'plugin') return ''
  return pluginStore.homePanelMode.startsWith('plugin:')
    ? pluginStore.homePanelMode
    : ''
})

/** 切换模式 */
async function onModeChange(value: string | number) {
  const v = String(value) as 'default' | 'plugin' | 'custom'
  if (v === 'default') {
    await pluginStore.setHomePanelMode('default')
  } else if (v === 'custom') {
    await pluginStore.setHomePanelMode('custom')
  } else if (v === 'plugin') {
    // 切到插件模式时，如果当前不是 plugin: 开头，选第一个可用插件
    if (!pluginStore.homePanelMode.startsWith('plugin:')) {
      const first = pluginOptions.value[0]
      if (first) {
        await pluginStore.setHomePanelMode(first.value as `plugin:${string}`)
      } else {
        await pluginStore.setHomePanelMode('default')
        toastInfo('暂无可用插件，已回退到默认模式')
      }
    }
  }
}

/** 切换选中的插件 */
async function onPluginSelect(value: string | number) {
  await pluginStore.setHomePanelMode(String(value) as `plugin:${string}`)
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">主页</h3>
    <div class="divide-y divide-gray-200">
      <!-- 模式选择 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">右侧内容区模式</p>
            <p class="text-xs text-gray-500 mt-0.5">
              选择主页右侧内容的显示方式：默认启动日志、已启用的插件、或自定义布局
            </p>
          </div>
          <div class="flex-none w-48">
            <Select
              :model-value="panelMode"
              :options="modeOptions"
              @update:model-value="onModeChange"
            />
          </div>
        </div>
      </div>

      <!-- 插件模式：选择插件 -->
      <div v-if="panelMode === 'plugin'" class="px-5 py-4 bg-gray-50/50">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">选择插件</p>
            <p class="text-xs text-gray-500 mt-0.5">
              仅显示已启用且提供主页内容区的插件；启用插件请前往「设置 → 插件」
            </p>
          </div>
          <div class="flex-none w-48">
            <Select
              v-if="pluginOptions.length > 0"
              :model-value="selectedPlugin"
              :options="pluginOptions"
              @update:model-value="onPluginSelect"
            />
            <p v-else class="text-xs text-gray-400">暂无可用插件</p>
          </div>
        </div>
      </div>

      <!-- 自定义模式：布局配置 -->
      <div v-else-if="panelMode === 'custom'" class="px-5 py-4 bg-gray-50/50">
        <CustomLayoutSection />
      </div>
    </div>
  </div>
</template>
