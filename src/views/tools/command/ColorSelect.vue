<script setup lang="ts">
/**
 * 颜色下拉选择：中文标签 + 圆形色块预览，封装公共 Select
 */
import { computed, defineAsyncComponent } from 'vue'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
import { COLOR_OPTIONS } from './generator'

const model = defineModel<string>('modelValue', { required: true })

const selected = computed(() => COLOR_OPTIONS.find((o) => o.value === model.value))
</script>

<template>
  <Select v-model="model" :options="COLOR_OPTIONS">
    <template #selected>
      <span v-if="selected" class="flex min-w-0 items-center gap-1.5">
        <span
          class="inline-block h-3 w-3 flex-shrink-0 rounded-full border border-black/10"
          :style="{ backgroundColor: selected.color }"
        />
        <span class="truncate">{{ selected.label }}</span>
      </span>
    </template>
    <template #option="{ option, selected: isSelected }">
      <span class="flex min-w-0 items-center gap-1.5">
        <span
          class="inline-block h-3 w-3 flex-shrink-0 rounded-full border border-black/10"
          :style="{ backgroundColor: option.color }"
        />
        <span class="flex-1 truncate">{{ option.label }}</span>
      </span>
      <svg v-if="isSelected" class="h-3 w-3 flex-shrink-0 text-primary-500" viewBox="0 0 1024 1024" fill="currentColor">
        <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
      </svg>
    </template>
  </Select>
</template>
