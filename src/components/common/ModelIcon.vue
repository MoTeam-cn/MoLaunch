<script setup lang="ts">
/**
 * 大模型品牌图标（彩色 / 黑白双模式）
 *
 * 依据模型名识别品牌（见 utils/model-icon.ts），渲染对应品牌的官方 SVG 图标；
 * 图标资源来自 @lobehub/icons-static-svg（utils/model-brand-icons.ts 静态映射库）。
 * - 彩色模式（默认）：品牌彩色 SVG 源码内联渲染（`<img>` 渲染 SVG 渐变会失效）；
 *   无官方彩色变体的品牌退回单色。
 * - 黑白模式：单色 SVG 经 `?url` 打包后以 `<img>` 渲染。
 * 未识别到品牌时统一兜底 HuggingFace 图标。
 * 显示模式由设置页「模型图标」控制（utils/model-icon-mode.ts 全局响应式）。
 * 尺寸由调用方通过 `class="w-4 h-4"` 等控制。
 */
import { computed } from 'vue'
import { resolveModelBrand, resolveModelColorRaw, resolveModelIconUrl } from '@/utils/model-icon'
import { iconColorMode } from '@/utils/model-icon-mode'
import { HUGGINGFACE_COLOR_RAW, HUGGINGFACE_MONO_URL } from '@/utils/model-brand-icons'

const props = defineProps<{
  /** 模型名称（如 `qwen2.5:32b`、`deepseek-r1`） */
  model?: string | null
}>()

const brand = computed(() => resolveModelBrand(props.model))
/** 黑白模式图标 URL（未识别品牌时兜底 HuggingFace 单色图） */
const monoUrl = computed(() => resolveModelIconUrl(props.model) ?? HUGGINGFACE_MONO_URL)
/** 彩色模式内联源码（未识别品牌时兜底 HuggingFace 彩色图） */
const colorRaw = computed(() => resolveModelColorRaw(props.model) ?? HUGGINGFACE_COLOR_RAW)
/** 是否内联渲染彩色图：彩色模式且该品牌有官方彩色变体（无品牌时兜底 HuggingFace 彩色图） */
const renderColor = computed(
  () =>
    iconColorMode.value === 'color' &&
    (brand.value ? resolveModelColorRaw(props.model) !== null : true),
)
</script>

<template>
  <span class="inline-flex items-center justify-center shrink-0" aria-hidden="true">
    <!-- eslint-disable-next-line vue/no-v-html -- colorRaw 为静态品牌图标资源，无用户输入 -->
      <span v-if="renderColor" class="model-icon-raw w-full h-full" v-html="colorRaw" />
    <img v-else :src="monoUrl" alt="" class="w-full h-full object-contain" />
  </span>
</template>

<style scoped>
/* 内联彩色 SVG：铺满容器（覆盖源文件 width/height="1em"） */
.model-icon-raw :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>
