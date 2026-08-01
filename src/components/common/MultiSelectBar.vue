<script setup lang="ts">
/**
 * 通用多选操作栏（浮动卡片）
 *
 * - 浮动在视口底部中央
 * - 不占据列表布局空间，通过 fixed 定位悬浮于内容之上
 * - 卡片分两部分：上方居中"已选择 X 项"文字，下方水平排列操作按钮
 * - 入场动画：从下方 10px 滑入 + 淡入
 *
 * 通用化设计：
 * - actions prop 接收操作按钮配置数组，每个按钮包含 key/label/icon/variant
 * - 点击按钮 emit action 事件，携带 key，由调用方分发到具体 handler
 * - 不包含任何业务逻辑，可复用于 Mod 列表、资源列表、下载列表等场景
 *
 * 使用方式：
 * ```vue
 * <MultiSelectBar
 *   :selected-count="selectedCount"
 *   :total-count="list.length"
 *   :actions="actions"
 *   :batch-processing="batchProcessing"
 *   @action="handleAction"
 *   @select-all="selectAll"
 *   @invert-selection="invertSelection"
 *   @exit="clearSelection"
 * />
 * ```
 */
import { computed, type Component } from 'vue'
import {
  CheckCircleIcon,
  XMarkIcon,
  Squares2X2Icon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'

/** 操作按钮变体样式 */
type ActionVariant = 'enable' | 'disable' | 'update' | 'delete' | 'default'

/** 批量操作按钮配置 */
export interface MultiSelectAction {
  /** 唯一标识，通过 action 事件传回调用方 */
  key: string
  /** 按钮文字 */
  label: string
  /** 图标组件（heroicons 等） */
  icon?: Component
  /** 颜色变体 */
  variant?: ActionVariant
  /** 是否禁用（如没有可更新项时禁用"更新"按钮） */
  disabled?: boolean
}

interface Props {
  /** 已选中的数量 */
  selectedCount: number
  /** 总数量（用于全选判断） */
  totalCount: number
  /** 批量操作按钮配置 */
  actions: MultiSelectAction[]
  /** 是否正在执行批量操作 */
  batchProcessing?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  batchProcessing: false,
})

const emit = defineEmits<{
  /** 点击操作按钮，携带 action key */
  action: [key: string]
  /** 全选/取消全选 */
  selectAll: []
  /** 反选 */
  invertSelection: []
  /** 退出多选（清空选中） */
  exit: []
}>()

const isAllSelected = computed(() => props.selectedCount === props.totalCount && props.totalCount > 0)

/** 操作按钮颜色变体配色 */
const variantClasses: Record<ActionVariant, string> = {
  enable: 'text-green-600 hover:bg-green-50',
  disable: 'text-yellow-600 hover:bg-yellow-50',
  update: 'text-blue-600 hover:bg-blue-50',
  delete: 'text-red-600 hover:bg-red-50',
  default: 'text-gray-600 hover:bg-gray-100',
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 translate-y-3"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-3"
    >
      <!-- 浮动卡片：视口底部中央 -->
      <div
        v-if="selectedCount > 0"
        class="fixed bottom-6 left-1/2 -translate-x-1/2 z-40 min-w-[320px]"
      >
        <div class="bg-white rounded-xl shadow-xl border border-gray-200 px-3 py-2">
          <!-- 上方：居中显示选中计数 -->
          <div class="flex items-center justify-center gap-1.5 text-xs text-gray-500 mb-1.5">
            <CheckCircleIcon class="w-3.5 h-3.5 text-blue-500" />
            <span class="font-medium">已选择 {{ selectedCount }} 项</span>
          </div>
          <!-- 下方：操作按钮行 -->
          <div class="flex items-center gap-0.5">
            <!-- 批量操作按钮（由调用方通过 actions prop 配置） -->
            <Tooltip
              v-for="action in actions"
              :key="action.key"
              :text="action.label"
              position="top"
            >
              <Button
type="ghost" size="small"
                class="px-2.5 py-1.5 text-xs font-medium rounded-md border-0 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                :class="variantClasses[action.variant || 'default']"
                :disabled="batchProcessing || selectedCount === 0 || action.disabled"
                @click="emit('action', action.key)"
              >
                <component :is="action.icon" v-if="action.icon" class="w-3.5 h-3.5" />
                {{ action.label }}
              </Button>
            </Tooltip>
            <!-- 分隔线 -->
            <div class="w-px h-5 bg-gray-200 mx-1" />
            <!-- 全选/反选 -->
            <Tooltip :text="isAllSelected ? '取消全选' : '全选'" position="top">
              <Button
type="ghost" size="small"
                class="px-2.5 py-1.5 text-xs font-medium rounded-md border-0 text-gray-600 hover:bg-gray-100 transition-colors"
                @click="emit('selectAll')"
              >
                <Squares2X2Icon class="w-3.5 h-3.5" />
                {{ isAllSelected ? '取消全选' : '全选' }}
              </Button>
            </Tooltip>
            <Tooltip text="反选" position="top">
              <Button
type="ghost" size="small"
                class="px-2.5 py-1.5 text-xs font-medium rounded-md border-0 text-gray-600 hover:bg-gray-100 transition-colors"
                @click="emit('invertSelection')"
              >
                <ArrowTopRightOnSquareIcon class="w-3.5 h-3.5" />
                反选
              </Button>
            </Tooltip>
            <!-- 分隔线 -->
            <div class="w-px h-5 bg-gray-200 mx-1" />
            <!-- 退出多选 -->
            <Tooltip text="退出多选（ESC）" position="top">
              <Button
type="ghost" size="small"
                class="px-2.5 py-1.5 text-xs font-medium rounded-md border-0 text-gray-600 hover:bg-gray-100 transition-colors"
                @click="emit('exit')"
              >
                <XMarkIcon class="w-3.5 h-3.5" />
                取消选择
              </Button>
            </Tooltip>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
