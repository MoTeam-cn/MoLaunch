<script setup lang="ts">
/**
 * 自定义背景层
 *
 * 固定在视口最底层的背景图 + 遮罩；样式完全由 <html> 上的 CSS 变量驱动
 * （--ui-bg-image / --ui-bg-blur / --ui-bg-mask / --ui-bg-size），
 * 因此切换设置时无需重新渲染组件。
 */
import { computed } from 'vue'
import { useUiPrefsStore } from '@/stores/ui-prefs'

const uiStore = useUiPrefsStore()
const visible = computed(() => uiStore.hasBackground)
</script>

<template>
  <div v-if="visible" class="pointer-events-none fixed inset-0 overflow-hidden" style="z-index: -1" aria-hidden="true">
    <div class="ui-bg-image absolute inset-0" />
    <div class="ui-bg-mask absolute inset-0" />
  </div>
</template>
