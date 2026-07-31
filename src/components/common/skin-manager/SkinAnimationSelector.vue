<script setup lang="ts">
/**
 * 皮肤 3D 预览动画状态选择器
 *
 * - 9 种动画状态（站立/行走/跑步/飞行/挥手/蹲下/受击/游泳/静止）
 * - 图标用 SVG path 数组（24x24 viewBox，stroke 风格）
 * - v-model 双向绑定当前动画
 */
import Tooltip from '../Tooltip.vue'

export type AnimationType = 'idle' | 'walk' | 'run' | 'fly' | 'wave' | 'crouch' | 'hit' | 'swim' | 'none'

const props = defineProps<{ modelValue: AnimationType }>()
const emit = defineEmits<{ 'update:modelValue': [AnimationType] }>()

/** 动画选项：图标用 SVG path 数组（24x24 viewBox，stroke 风格） */
const animationOptions: { value: AnimationType; label: string; paths: string[] }[] = [
  { value: 'idle', label: '站立', paths: ['M12 2a2 2 0 100 4 2 2 0 000-4z', 'M12 6v10', 'M9 9l3-3 3 3', 'M9 20l3-4 3 4'] },
  { value: 'walk', label: '行走', paths: ['M13 4a2 2 0 100 4 2 2 0 000-4z', 'M13 8v6', 'M13 11l-3 2', 'M13 11l3 2', 'M13 14l-2 5', 'M13 14l2 5'] },
  { value: 'run', label: '跑步', paths: ['M14 3a2 2 0 100 4 2 2 0 000-4z', 'M14 7v6', 'M14 9l-4 1', 'M14 9l4 1', 'M14 13l-3 6', 'M14 13l3 6', 'M5 20h4'] },
  { value: 'fly', label: '飞行', paths: ['M12 2a2 2 0 100 4 2 2 0 000-4z', 'M12 6v8', 'M4 10l8-2 8 2', 'M9 20l3-6 3 6'] },
  { value: 'wave', label: '挥手', paths: ['M12 5a2 2 0 100 4 2 2 0 000-4z', 'M12 9v8', 'M12 11l-4-2', 'M12 12l4-2', 'M9 19l3-2 3 2'] },
  { value: 'crouch', label: '蹲下', paths: ['M12 4a2 2 0 100 4 2 2 0 000-4z', 'M12 8v5', 'M9 13h6', 'M8 13v5', 'M16 13v5'] },
  { value: 'hit', label: '受击', paths: ['M12 3a2 2 0 100 4 2 2 0 000-4z', 'M12 7v8', 'M9 10l-3-1', 'M15 10l3-1', 'M9 20l3-5 3 5'] },
  { value: 'swim', label: '游泳', paths: ['M5 8a2 2 0 100 4 2 2 0 000-4z', 'M3 14h4l4-2 4 2 4-1 2 1', 'M3 18h4l4-1 4 1 4-1 2 1'] },
  { value: 'none', label: '静止', paths: ['M12 3a2 2 0 100 4 2 2 0 000-4z', 'M12 7v10', 'M12 7l-3 3', 'M12 7l3 3', 'M9 21h6'] },
]
</script>

<template>
  <div class="rounded-lg border border-gray-100 p-4">
    <div class="mb-3 text-sm font-medium text-gray-700">动画状态</div>
    <div class="flex flex-wrap gap-1.5">
      <Tooltip
        v-for="opt in animationOptions"
        :key="opt.value"
        :text="opt.label"
        position="top"
        :delay="0"
      >
        <!-- 保留原生 button：动画选择图标按钮（h-8 w-8 + border + active 状态），
             Button.vue 的 scoped size 类固定 height/padding 会破坏紧凑网格布局 -->
        <button
          class="flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
          :class="props.modelValue === opt.value
            ? 'border-primary-500 bg-primary-50 text-primary-700'
            : 'border-gray-200 text-gray-500 hover:bg-gray-50'"
          @click="emit('update:modelValue', opt.value)"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path v-for="(d, i) in opt.paths" :key="i" :d="d" />
          </svg>
        </button>
      </Tooltip>
    </div>
  </div>
</template>
