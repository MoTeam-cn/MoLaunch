<script setup lang="ts">
/**
 * 前置 Mod 确认弹窗
 *
 * 下载主 Mod 前调用 check_mod_dependencies，发现缺失前置时弹出此窗：
 * - 列出缺失前置，用户可勾选要一并安装的项（默认全选有兼容版本的）
 * - 顶部全选/反选，底部"确认安装"按钮触发 confirm 事件
 * - 已安装前置（upToDate）以折叠区形式展示，便于核对
 *
 * 调用方：ResourceDetail.vue 的 handleDownload 流程
 */
import { computed, ref, watch } from 'vue'
import type { ResolvedDependency } from '@/types/community'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import DependencyItem from './DependencyItem.vue'
import {
  XMarkIcon,
  CheckCircleIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 弹窗可见性 */
  visible: boolean
  /** 缺失的前置列表 */
  missing: ResolvedDependency[]
  /** 已满足的前置列表（折叠展示） */
  upToDate: ResolvedDependency[]
  /** 主 Mod 名称（标题展示用） */
  mainName: string
  /** 是否正在安装中（确认按钮 loading） */
  installing: boolean
  /** 是否正在检查中（首次进入时） */
  checking: boolean
}>()

const emit = defineEmits<{
  /** 关闭弹窗（取消） */
  close: []
  /** 确认安装，回传用户勾选的前置列表（空数组表示仅装主 mod） */
  confirm: [deps: ResolvedDependency[]]
}>()

// 用户勾选的依赖项 key 集合（key = project.id）
const selectedKeys = ref<Set<string>>(new Set())

// 已满足区折叠状态（默认折叠，减少视觉负担）
const upToDateExpanded = ref(false)

// 是否展示已满足区
const hasUpToDate = computed(() => props.upToDate.length > 0)

// 可勾选的依赖（有兼容版本才能勾）
const selectableDeps = computed(() => props.missing.filter(d => d.suggestedVersion !== null))

// 全选状态（所有可勾选的都被选中才算全选）
const allSelected = computed(
  () => selectableDeps.value.length > 0
    && selectableDeps.value.every(d => selectedKeys.value.has(d.project.id)),
)

// 选中数量（用于底部按钮文案）
const selectedCount = computed(() => {
  let n = 0
  for (const d of props.missing) {
    if (selectedKeys.value.has(d.project.id)) n++
  }
  return n
})

// missing 变化时重置勾选（默认全选可勾选项）
watch(
  () => props.missing,
  (list) => {
    selectedKeys.value = new Set(
      list.filter(d => d.suggestedVersion !== null).map(d => d.project.id),
    )
  },
  { immediate: true },
)

function toggle(id: string) {
  const next = new Set(selectedKeys.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedKeys.value = next
}

function toggleAll() {
  if (allSelected.value) {
    selectedKeys.value = new Set()
  } else {
    selectedKeys.value = new Set(selectableDeps.value.map(d => d.project.id))
  }
}

function handleConfirm() {
  const selected = props.missing.filter(d => selectedKeys.value.has(d.project.id))
  emit('confirm', selected)
}

function handleClose() {
  if (props.installing) return // 安装中禁止关闭
  emit('close')
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[10003] flex items-start justify-center px-4 pt-14 pb-4"
        @click.self="handleClose"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-lg bg-white rounded-lg shadow-xl flex flex-col max-h-[calc(100vh-100px)] mt-2">
          <!-- 头部 -->
          <div class="px-5 py-3 border-b border-gray-100 flex items-center justify-between">
            <div class="flex items-center gap-2 min-w-0">
              <CheckCircleIcon class="w-5 h-5 text-primary-500 shrink-0" />
              <div class="min-w-0">
                <div class="text-sm font-semibold text-gray-900 truncate">
                  发现 {{ missing.length }} 个缺失前置
                </div>
                <div class="text-xs text-gray-500 truncate">主 Mod：{{ mainName }}</div>
              </div>
            </div>
            <!-- 保留原生 button：图标关闭按钮(w-7 h-7)/文本链接/折叠头均为自定义尺寸或布局，
                 Button.vue 的 scoped size 类固定 height/padding 无法被工具类覆盖 -->
            <button
              type="button"
              class="w-7 h-7 flex items-center justify-center rounded-md text-gray-400 hover:bg-gray-100 hover:text-gray-600 transition-colors shrink-0"
              :disabled="installing"
              @click="handleClose"
            >
              <XMarkIcon class="w-4 h-4" />
            </button>
          </div>

          <!-- 检查中 -->
          <div v-if="checking" class="flex-1 flex flex-col items-center justify-center py-12">
            <ArrowPathIcon class="w-6 h-6 text-primary-400 animate-spin mb-3" />
            <span class="text-sm text-gray-500">正在检查前置依赖...</span>
          </div>

          <!-- 列表区 -->
          <div v-else class="flex-1 overflow-y-auto px-4 py-3 space-y-2">
            <!-- 全选/反选 -->
            <div class="flex items-center justify-between px-1">
              <button
                type="button"
                class="text-xs text-primary-600 hover:text-primary-700 font-medium"
                @click="toggleAll"
              >
                {{ allSelected ? '取消全选' : '全选可安装项' }}
              </button>
              <span class="text-xs text-gray-400">
                已选 {{ selectedCount }} / {{ missing.length }} 项
              </span>
            </div>

            <!-- 缺失前置列表 -->
            <div class="space-y-1.5">
              <DependencyItem
                v-for="dep in missing"
                :key="dep.project.id"
                :dep="dep"
                :selected="selectedKeys.has(dep.project.id)"
                @toggle="toggle(dep.project.id)"
              />
            </div>

            <!-- 已满足前置（折叠区） -->
            <div v-if="hasUpToDate" class="pt-2 border-t border-gray-100">
              <button
                type="button"
                class="w-full flex items-center justify-between px-1 py-1 text-xs text-gray-500 hover:text-gray-700"
                @click="upToDateExpanded = !upToDateExpanded"
              >
                <span>已安装或满足的前置：{{ upToDate.length }} 项</span>
                <span class="transition-transform duration-200" :class="upToDateExpanded ? 'rotate-180' : ''">▾</span>
              </button>
              <div v-if="upToDateExpanded" class="mt-1.5 space-y-1.5">
                <DependencyItem
                  v-for="dep in upToDate"
                  :key="dep.project.id"
                  :dep="dep"
                  :selected="true"
                />
              </div>
            </div>
          </div>

          <!-- 底部操作 -->
          <div class="px-5 py-3 border-t border-gray-100 flex items-center justify-end gap-2">
            <Tooltip v-if="installing" text="安装进行中，请稍候..." position="top">
              <Button type="text" size="small" disabled>取消</Button>
            </Tooltip>
            <Button v-else type="text" size="small" @click="handleClose">取消</Button>
            <Button
              type="primary"
              size="small"
              :loading="installing"
              @click="handleConfirm"
            >
              {{ selectedCount > 0 ? `安装主 Mod + ${selectedCount} 个前置` : '仅安装主 Mod' }}
            </Button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
