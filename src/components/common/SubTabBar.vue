<script setup lang="ts">
/**
 * 顶部子菜单切换组件
 *
 * 用于页面内的子页签切换（如：关于 / 鸣谢 / 教程）。
 * 支持 sticky 固定模式，滚动时子菜单栏吸顶。
 *
 * 用法：
 * <SubTabBar v-model="activeTab" :tabs="tabs" sticky />
 */
interface Tab {
  id: string
  label: string
  icon?: any
}

interface Props {
  tabs: Tab[]
  modelValue: string
  sticky?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  sticky: false,
})
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

function selectTab(id: string) {
  emit('update:modelValue', id)
}
</script>

<template>
  <div
    class="flex items-center gap-1 border-b border-gray-200 bg-white px-1"
    :class="sticky ? 'sticky top-0 z-20 shadow-sm' : ''"
  >
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="relative flex items-center gap-1.5 px-4 py-2.5 text-[13px] font-medium transition-colors"
      :class="modelValue === tab.id
        ? 'text-primary-600'
        : 'text-gray-500 hover:text-gray-700'"
      @click="selectTab(tab.id)"
    >
      <component :is="tab.icon" v-if="tab.icon" class="h-4 w-4" />
      {{ tab.label }}
      <!-- 底部选中指示线 -->
      <span
        v-if="modelValue === tab.id"
        class="absolute bottom-0 left-2 right-2 h-0.5 rounded-full bg-primary-500"
      />
    </button>
  </div>
</template>
