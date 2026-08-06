<script setup lang="ts">
/**
 * 步骤进度条公共组件
 *
 * 用于展示多步骤流程的当前进度，配合伪进度动画提升等待体验。
 * 典型场景：微软登录 Token 交换（XBL → XSTS → MC Token → ...）、启动流程等。
 *
 * 特性：
 * - 顶部进度条：真实进度（currentIndex / steps.length）+ 伪进度动画（在等待时缓慢爬升，数字保持变动）
 * - 步骤列表：已完成（绿勾）/ 当前（高亮脉冲）/ 未完成（灰点）
 * - "突然涨上去"的动态效果：伪进度用 CSS 过渡（宽度变化带 transition）
 */
import { computed, ref, watch, onUnmounted } from 'vue'
import { CheckIcon } from '@heroicons/vue/24/solid'

export interface ProgressStep {
  key: string
  label: string
}

const props = withDefaults(
  defineProps<{
    /** 步骤列表 [{ key, label }] */
    steps: readonly ProgressStep[]
    /** 当前步骤下标（-1 = 尚未开始/前置阶段） */
    currentIndex?: number
    /** 是否显示百分比数字（默认 true） */
    showPercent?: boolean
    /** 是否显示步骤列表（默认 true） */
    showSteps?: boolean
    /** 是否叠加扫光层（默认 false，不影响微软登录等现有使用方；样式由 main.css 的 .progress-sweep 提供） */
    sweep?: boolean
  }>(),
  { currentIndex: -1, showPercent: true, showSteps: true, sweep: false },
)

const currentIndex = computed(() => props.currentIndex)

/** 是否已到最后一个步骤（视为完成，进度置 100%） */
const isDone = computed(
  () => props.steps.length > 0 && currentIndex.value >= props.steps.length - 1,
)

/** 真实进度（基于步骤下标）：已完成步骤数 / 总步骤数 */
const realPercent = computed(() => {
  if (props.steps.length === 0) return 0
  if (isDone.value) return 100
  const done = Math.max(0, currentIndex.value)
  return Math.round((done / props.steps.length) * 100)
})

// 伪进度：在等待当前步骤完成时，从真实进度缓慢向 100% 爬升
const fakePercent = ref(realPercent.value)
let fakeTimer: ReturnType<typeof setInterval> | null = null

function startFakeProgress() {
  stopFakeProgress()
  fakeTimer = setInterval(() => {
    // 目标：真实进度 + 尚未跨越的剩余部分（最多到 95，避免假满）
    // 真实进度附近缓步爬升（等待当前步骤），真实进度跳变时 CSS transition 呈现"突然涨上去"
    const target = Math.min(95, realPercent.value + (100 - realPercent.value) * 0.25)
    if (fakePercent.value < target) {
      fakePercent.value += Math.max(0.15, (target - fakePercent.value) * 0.03)
      if (fakePercent.value > target) fakePercent.value = target
    }
  }, 150)
}

function stopFakeProgress() {
  if (fakeTimer) { clearInterval(fakeTimer); fakeTimer = null }
}

// 真实进度变化时，伪进度跟随（当前步骤跳转时有"突然涨上去"的动态效果）
watch(realPercent, (v) => {
  fakePercent.value = Math.max(v, fakePercent.value)
  if (v >= 95) { fakePercent.value = 100; stopFakeProgress() }
  else { startFakeProgress() }
})

// 挂载即启动伪进度爬升（交换阶段开始即有进度在动），完成态不启动
if (realPercent.value < 95) {
  fakePercent.value = realPercent.value
  startFakeProgress()
}

onUnmounted(stopFakeProgress)
</script>

<template>
  <div class="space-y-3">
    <!-- 顶部进度条（伪进度动画） -->
    <div class="flex items-center gap-3">
      <div class="relative h-2 flex-1 overflow-hidden rounded-full bg-gray-100">
        <div
          class="h-full rounded-full bg-primary-500 transition-all duration-500 ease-out"
          :style="{ width: fakePercent + '%' }"
        />
        <!-- 伪进度移动的高光 -->
        <div
          v-if="fakePercent < 100"
          class="absolute top-0 h-full w-1/3 bg-white/40 blur-sm"
          :style="{ left: fakePercent + '%' }"
        />
        <!-- 扫光层（sweep=true 时叠加；动画由 main.css 的 .progress-sweep 定义） -->
        <div v-if="sweep" class="progress-sweep" />
      </div>
      <span v-if="showPercent" class="w-10 shrink-0 text-right text-xs font-semibold tabular-nums text-primary-600">
        {{ Math.round(fakePercent) }}%
      </span>
    </div>

    <!-- 步骤列表 -->
    <div v-if="showSteps" class="space-y-1.5">
      <div
        v-for="(s, idx) in steps"
        :key="s.key"
        class="flex items-center gap-3 rounded-lg px-3 py-1.5 text-sm transition-colors"
        :class="{
          'bg-primary-50 text-primary-700': idx === currentIndex,
          'text-gray-400': idx > currentIndex,
          'text-green-600': idx >= 0 && idx < currentIndex,
        }"
      >
        <CheckIcon v-if="idx >= 0 && idx < currentIndex" class="h-4 w-4 text-green-500" />
        <span v-else-if="idx === currentIndex" class="h-2 w-2 animate-pulse rounded-full bg-primary-500" />
        <span v-else class="h-2 w-2 rounded-full bg-gray-300" />
        <span>{{ s.label }}</span>
      </div>
    </div>
  </div>
</template>
