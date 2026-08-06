<script setup lang="ts">
/**
 * 分析阶段指示条（与 Input/Button 同设计语言：灰底无边框 + 圆角 + text-xs）
 *
 * 逻辑约定：
 * - 思考阶段（尚未收到正文 STEP 标记）：不显示任何阶段，只显示「正在思考如何判断问题…」
 *   灰底不可用状态，省略号从前到后有规律地增长动画（. → .. → ... → 循环）——阶段进度
 *   对思考过程无意义，绝不伪进度。
 * - 正文输出阶段：由模型输出的【STEP:N/5】标记驱动，显示当前环节（primary 蓝标签），
 *   已完成环节显示绿勾。不做百分比/伪进度动画。
 */
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { CheckIcon } from '@heroicons/vue/24/solid'

export interface AnalyzeStage {
  key: string
  label: string
}

const props = withDefaults(
  defineProps<{
    /** 环节列表 [{ key, label }] */
    stages: readonly AnalyzeStage[]
    /** 当前环节下标（-1 = 思考阶段，未输出正文 STEP 标记） */
    currentIndex?: number
  }>(),
  { currentIndex: -1 },
)

/** 是否已进入正文输出阶段（收到过 STEP 标记） */
const inOutput = computed(() => props.currentIndex >= 0 && props.currentIndex < props.stages.length)

// 思考阶段省略号动画：0→1→2→3 个点，有规律缓慢循环（约 1.2s 一轮）
const dotCount = ref(0)
let dotTimer: ReturnType<typeof setInterval> | null = null

function startDotTimer() {
  stopDotTimer()
  dotCount.value = 0
  dotTimer = setInterval(() => {
    dotCount.value = (dotCount.value + 1) % 4
  }, 400)
}

function stopDotTimer() {
  if (dotTimer) {
    clearInterval(dotTimer)
    dotTimer = null
  }
}

onMounted(() => {
  if (!inOutput.value) startDotTimer()
})
onUnmounted(stopDotTimer)

// 从思考阶段进入正文输出时停止动画；反之（重新分析回到思考）重新开始
watch(inOutput, (v) => {
  if (v) stopDotTimer()
  else startDotTimer()
})
</script>

<template>
  <div class="flex items-center gap-2">
    <!-- 思考阶段：不可用状态，不显示阶段 -->
    <span
      v-if="!inOutput"
      class="inline-flex items-center gap-1.5 rounded-md bg-gray-100 px-2.5 py-1 text-xs text-gray-500"
    >
      <span class="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-primary-500" />
      正在思考如何判断问题{{ '.'.repeat(dotCount) }}
    </span>

    <!-- 正文输出阶段：由 STEP 驱动 -->
    <div v-else class="flex items-center gap-1">
      <span
        v-for="(s, idx) in stages"
        :key="s.key"
        class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium"
        :class="{
          'bg-primary-50 text-primary-600': idx === currentIndex,
          'text-gray-300': idx > currentIndex,
          'text-green-600': idx < currentIndex,
        }"
      >
        <CheckIcon v-if="idx < currentIndex" class="h-3 w-3" />
        <span v-else-if="idx === currentIndex" class="inline-block h-1 w-1 rounded-full bg-primary-500" />
        <span v-else class="inline-block h-1 w-1 rounded-full bg-gray-200" />
        {{ s.label }}
      </span>
    </div>
  </div>
</template>
