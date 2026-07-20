<script setup lang="ts">
/**
 * 按钮组件（复刻 Arco Design Button 样式）
 *
 * 类型：
 * - primary：蓝底白字（默认）
 * - secondary：灰底深字
 * - outline：透明底蓝边框
 * - ghost：透明底，hover 浅灰
 * - text：文本按钮
 *
 * 尺寸：mini(24px) / small(28px) / default(32px) / large(36px)
 *
 * 用法：
 * <Button type="primary" @click="...">确定</Button>
 * <Button type="secondary" :loading="true">加载中</Button>
 * <Button type="outline" size="small">
 *   <template #icon><PlusIcon /></template>
 *   添加
 * </Button>
 */
import { computed } from 'vue'

interface Props {
  /** 按钮类型 */
  type?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'text'
  /** 按钮尺寸 */
  size?: 'mini' | 'small' | 'default' | 'large'
  /** 是否禁用 */
  disabled?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** 是否撑满父容器宽度 */
  long?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'primary',
  size: 'default',
  disabled: false,
  loading: false,
  long: false,
})

defineEmits<{ click: [e: MouseEvent] }>()

const sizeClass = computed(() => `btn-size-${props.size}`)
</script>

<template>
  <button
    :class="[
      'btn',
      `btn-${type}`,
      sizeClass,
      { 'btn-long': long, 'btn-loading': loading },
    ]"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <!-- 加载图标 -->
    <svg v-if="loading" class="btn-spinner" viewBox="0 0 1024 1024" fill="currentColor">
      <path d="M512 64a448 448 0 1 0 448 448 32 32 0 0 0-64 0 384 384 0 1 1-384-384 32 32 0 0 0 0-64z" />
    </svg>
    <!-- 前置图标 -->
    <slot v-else name="icon" />
    <!-- 文字内容 -->
    <slot />
  </button>
</template>

<style scoped>
/* 按钮基础样式已在全局 main.css 中定义（.btn / .btn-primary 等）
   这里只补充尺寸和特殊状态 */

/* 尺寸 */
.btn-size-mini {
  height: 24px;
  padding: 0 11px;
  font-size: 12px;
}
.btn-size-small {
  height: 28px;
  padding: 0 15px;
  font-size: 14px;
}
.btn-size-default {
  height: 32px;
  padding: 0 15px;
  font-size: 14px;
}
.btn-size-large {
  height: 36px;
  padding: 0 19px;
  font-size: 14px;
}

/* 撑满宽度 */
.btn-long {
  display: flex;
  width: 100%;
}

/* 图标与文字间距（有图标时） */
.btn:not(.btn-size-mini) > :deep(svg:not(.btn-spinner)) {
  margin-right: 8px;
}
.btn-size-mini > :deep(svg:not(.btn-spinner)) {
  margin-right: 4px;
}
.btn-size-small > :deep(svg:not(.btn-spinner)) {
  margin-right: 6px;
}

/* 没有 slot 文字内容时（纯图标按钮），去掉 margin */
.btn:empty > :deep(svg:not(.btn-spinner)) {
  margin-right: 0;
}

/* 加载动画 */
.btn-spinner {
  width: 14px;
  height: 14px;
  margin-right: 8px;
  animation: btn-spin 0.8s linear infinite;
}
.btn:empty .btn-spinner {
  margin-right: 0;
}

@keyframes btn-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* loading 状态下隐藏图标槽位 */
.btn-loading > :deep(slots[name='icon']) {
  display: none;
}
</style>
