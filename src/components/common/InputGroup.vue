<script setup lang="ts">
/**
 * 输入框组合布局
 *
 * 将多个 Input / Select 等表单控件按指定占比并列排列，
 * 基于 CSS Grid 实现，支持任意数量的子项与列间距配置。
 *
 * 用法：
 * <InputGroup :ratio="[3, 1]">
 *   <Input v-model="addr" placeholder="服务器地址" />
 *   <Input v-model="port" placeholder="端口" />
 * </InputGroup>
 *
 * 设计参考 Arco Design FormItem 的 inline 布局，
 * 仅负责网格容器，子项样式由各表单组件自身控制。
 */
import { computed } from 'vue'

interface Props {
  /** 各列占比（fr 单位），如 [3, 1] 表示 3:1，[1, 1, 1] 表示三等分 */
  ratio?: number[]
  /** 列间距，默认 12px（与 Input 组件内边距协调） */
  gap?: string
}

const props = withDefaults(defineProps<Props>(), {
  ratio: () => [1, 1],
  gap: '12px',
})

const gridColumns = computed(() => props.ratio.map(r => `${r}fr`).join(' '))
</script>

<template>
  <div class="input-group" :style="{ gridTemplateColumns: gridColumns, gap }">
    <slot />
  </div>
</template>

<style scoped>
.input-group {
  display: grid;
  align-items: center;
  width: 100%;
}
/* Input 组件根元素是 <span class="input-root"> 且 display: inline-block，
   在 grid 容器中可能未被正确 blockify，导致 grid item 退化为 inline-level
   上下堆叠。强制为 block 让 grid 列宽正确分配。 */
.input-group :deep(.input-root) {
  display: block;
  width: 100%;
}
</style>
