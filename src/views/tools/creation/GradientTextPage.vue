<script setup lang="ts">
/**
 * 渐变文字生成器（创作工具）
 *
 * 输入多行文本与颜色停靠点，预览 Minecraft 阴影渐变效果，
 * 生成 19 种输出格式（Vanilla / MiniMessage / JSON / BBCode 等）并支持复制/下载。
 *
 * 布局：文本编辑（左） + 输出配置（右），底部预设管理。
 */
import { reactive, watch, defineAsyncComponent } from 'vue'
import { PencilSquareIcon } from '@heroicons/vue/24/outline'
const GradientTextEditor = defineAsyncComponent(() => import('./gradient-text/GradientTextEditor.vue'))
const GradientColorStops = defineAsyncComponent(() => import('./gradient-text/GradientColorStops.vue'))
const GradientOutputPanel = defineAsyncComponent(() => import('./gradient-text/GradientOutputPanel.vue'))
const GradientPresetsManager = defineAsyncComponent(() => import('./gradient-text/GradientPresetsManager.vue'))
import { loadGradientTextState, saveGradientTextState } from '@/utils/gradient-text'
import type { GradientTextState } from '@/utils/gradient-text'

const state = reactive<GradientTextState>(loadGradientTextState())

watch(
  state,
  (value) => {
    saveGradientTextState({ ...value })
  },
  { deep: true },
)
</script>

<template>
  <section
    class="rounded-lg border border-gray-300 bg-white overflow-hidden"
  >
    <!-- 标题 -->
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <div class="flex items-center gap-2">
        <PencilSquareIcon class="h-5 w-5 text-gray-700" />
        <h3 class="text-sm font-semibold text-gray-900">渐变文字生成器</h3>
      </div>
    </div>

    <div class="grid gap-4 px-5 pb-5 lg:grid-cols-2">
      <!-- 左栏：文本编辑 + 颜色 -->
      <div class="space-y-4">
        <GradientTextEditor
          :document="state.document"
          @update:document="state.document = $event"
        />
        <GradientColorStops
          :colors="state.colors"
          @update:colors="state.colors = $event"
        />
      </div>

      <!-- 右栏：预览 + 输出 -->
      <GradientOutputPanel
        :document="state.document"
        :colors="state.colors"
        :adapter-id="state.adapterId"
        :vanilla-character="state.vanillaCharacter"
        :simplify-gradients="state.simplifyGradients"
        @update:adapter-id="state.adapterId = $event"
        @update:vanilla-character="state.vanillaCharacter = $event"
        @update:simplify-gradients="state.simplifyGradients = $event"
      />
    </div>

    <!-- 预设管理 -->
    <GradientPresetsManager
      :presets="state.presets"
      :colors="state.colors"
      @update:presets="state.presets = $event"
      @update:colors="state.colors = $event"
    />
  </section>
</template>