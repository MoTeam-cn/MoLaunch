<script setup lang="ts">
/**
 * 滑块组件（原生 <input type="range"> 封装）
 *
 * - modelValue: number      当前值（v-model）
 * - min / max / step        数值范围与步进
 * - marks                   档位标签 [{ value, label }]，渲染在轨道下方
 * - disabled                禁用（整体降透明度）
 *
 * 滑块与已填充轨道使用项目 primary 配色（var(--color-primary-500)）。
 */
import { computed, ref, watch } from 'vue'

interface SliderMark {
  value: number
  label: string
}

interface Props {
  modelValue: number
  min?: number
  max?: number
  step?: number
  marks?: SliderMark[]
  /** 吸附档位值列表：提供后拖动连续（step 内部强制 1，thumb 平滑跟随）、松手时吸附到最近档位再提交 */
  snap?: number[]
  disabled?: boolean
  /** 流星流光效果（Codex 风格紫色流星）：值达到高档（≥66%）时在已填充轨道上出现 */
  meteor?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
  marks: () => [],
  snap: () => [],
  disabled: false,
  meteor: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

function clamp(v: number): number {
  if (Number.isNaN(v)) return props.min
  return Math.min(props.max, Math.max(props.min, v))
}

/** 显示值：拖动中为连续临时值，否则跟随 modelValue */
const inner = ref(clamp(props.modelValue))
watch(
  () => props.modelValue,
  (v) => {
    inner.value = clamp(v)
  },
)

/** 拖动中：thumb 平滑跟随；非吸附模式实时提交，吸附模式松手时才提交 */
function onInput(v: number) {
  inner.value = clamp(v)
  if (!props.snap.length) emit('update:modelValue', clamp(v))
}

/** 松手：吸附到最近档位后提交 */
function onChange(v: number) {
  const n = clamp(v)
  if (props.snap.length) {
    const nearest = props.snap.reduce((best, cur) => (Math.abs(cur - n) < Math.abs(best - n) ? cur : best))
    inner.value = nearest
    emit('update:modelValue', nearest)
  } else {
    emit('update:modelValue', n)
  }
}

/** 当前值在轨道上的百分比（用于轨道填充渐变） */
const percent = computed(() =>
  props.max === props.min ? 0 : ((inner.value - props.min) / (props.max - props.min)) * 100,
)

function markPercent(mark: SliderMark): number {
  if (props.max === props.min) return 0
  return ((mark.value - props.min) / (props.max - props.min)) * 100
}

/** 首/尾标签用 0% / -100% 位移，避免半截溢出滑块外 */
function markTransform(index: number, count: number): string {
  if (index === 0) return 'translateX(0)'
  if (index === count - 1) return 'translateX(-100%)'
  return 'translateX(-50%)'
}

function markActive(mark: SliderMark): boolean {
  return clamp(mark.value) === inner.value
}

/** 轨道背景：已填充段 primary，未填充段浅灰 */
const trackStyle = computed(() => {
  const p = percent.value
  return {
    background: `linear-gradient(to right, var(--color-primary-500) 0%, var(--color-primary-500) ${p}%, var(--color-skeleton-bg, #e9e6ec) ${p}%, var(--color-skeleton-bg, #e9e6ec) 100%)`,
  }
})

/** 流星流光：仅当开启 meteor 且值达到高档（≥66%）时出现 */
const showMeteor = computed(() => props.meteor && percent.value >= 66)
</script>

<template>
  <div class="slider-root" :class="{ 'slider-disabled': disabled }">
    <div class="slider-track-wrap">
      <input
        class="slider-input"
        type="range"
        :min="min"
        :max="max"
        :step="snap.length ? 1 : step"
        :value="inner"
        :disabled="disabled"
        :style="trackStyle"
        aria-label="slider"
        @input="onInput(Number(($event.target as HTMLInputElement).value))"
        @change="onChange(Number(($event.target as HTMLInputElement).value))"
      />
      <!-- 流星流光层：仅覆盖已填充轨道宽度，内部扫光裁剪 -->
      <div v-if="showMeteor" class="slider-meteor" :style="{ width: percent + '%' }">
        <div class="progress-sweep" />
      </div>
    </div>
    <div v-if="marks.length > 0" class="slider-marks">
      <span
        v-for="(mark, i) in marks"
        :key="mark.value"
        class="slider-mark"
        :class="{ 'slider-mark-active': markActive(mark) }"
        :style="{ left: markPercent(mark) + '%', transform: markTransform(i, marks.length) }"
      >
        {{ mark.label }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.slider-root {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  min-width: 120px;
}

.slider-track-wrap {
  position: relative;
  /* 高度容纳 14px thumb，轨道线垂直居中，避免 thumb 溢出导致贴底/偏移 */
  height: 18px;
  display: flex;
  align-items: center;
}

.slider-input {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 4px;
  border-radius: 9999px;
  outline: none;
  cursor: pointer;
  margin: 0;
}

/* 流星流光层：覆盖已填充轨道（保持轨道线高度并垂直居中，不遮挡 thumb），overflow 裁剪使扫光只在填充段内可见 */
.slider-meteor {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  height: 4px;
  left: 0;
  overflow: hidden;
  border-radius: 9999px;
  pointer-events: none;
}

.slider-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #ffffff;
  border: 2px solid var(--color-primary-500);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  box-sizing: border-box;
}

.slider-input::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #ffffff;
  border: 2px solid var(--color-primary-500);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  box-sizing: border-box;
}

.slider-input::-moz-range-track {
  background: transparent;
}

/* 禁用态：整体降低透明度 */
.slider-root.slider-disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.slider-root.slider-disabled .slider-input {
  cursor: not-allowed;
}

.slider-marks {
  position: relative;
  height: 16px;
  margin-top: 8px;
  /* 标签不拦截鼠标：滑块可自由拖动 */
  pointer-events: none;
  user-select: none;
}

.slider-mark {
  position: absolute;
  top: 0;
  transform: translateX(-50%);
  font-size: 10px;
  color: #86909c;
  white-space: nowrap;
}

.slider-mark-active {
  color: var(--color-primary-500);
  font-weight: 500;
}
</style>