<script setup lang="ts">
/**
 * AI 工具链（消息列表内、AI 回复消息框上方）
 *
 * 极简收起条（默认收起）：一行「工具链 · N 个调用」。
 * 点击展开后以虚线时间线串联各工具调用（参考更新日志 ReleaseTimeline 样式）：
 * - 每个工具一个节点：左侧虚线 + 圆点 + 工具名 + 状态（执行中/完成）
 * - 点击节点展开入参（arguments）与执行输出（output），支持复制
 */
import { ref, watch } from 'vue'
import { CommandLineIcon, CheckCircleIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
import type { ToolCallItem } from '@/composables/useAiChat'
import { copyToClipboard } from '@/utils/clipboard'
import { toastSuccess, toastError } from '@/utils/toast'

const props = withDefaults(
  defineProps<{
    calls: ToolCallItem[]
    /** 传入 true 时自动展开工具链（对话进行中实时查看执行状态） */
    autoExpand?: boolean
  }>(),
  { autoExpand: false },
)

/** 整条工具链是否展开（默认收起） */
const expanded = ref(false)
/** 当前展开详情的工具节点索引（null 表示收起） */
const openIdx = ref<number | null>(null)

watch(
  () => props.autoExpand,
  (v) => {
    if (v) expanded.value = true
  },
  { immediate: true },
)

async function copy(text: string) {
  const ok = await copyToClipboard(text)
  if (ok) toastSuccess('已复制')
  else toastError('复制失败')
}

function toggleDetail(i: number) {
  openIdx.value = openIdx.value === i ? null : i
}
</script>

<template>
  <div class="tool-chain">
    <!-- 极简收起条：默认收起 -->
    <button
      type="button"
      class="flex items-center gap-1.5 rounded px-1.5 py-0.5 text-[11px] text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600"
      @click="expanded = !expanded"
    >
      <CommandLineIcon class="h-3.5 w-3.5" />
      <span>工具链 · {{ calls.length }} 个调用</span>
      <ChevronDownIcon class="h-3 w-3 transition-transform duration-200" :class="expanded ? 'rotate-180' : ''" />
    </button>

    <!-- 虚线时间线：串联各工具调用（整体淡入 + 节点逐个入场） -->
    <Transition name="chain">
      <TransitionGroup v-if="expanded" name="node" tag="ol" class="tool-timeline">
        <li v-for="(call, i) in calls" :key="call.index" class="tool-item">
          <span class="tool-dot" :class="call.status === 'done' ? 'is-done' : 'is-running'" />
          <div class="tool-body">
            <!-- 调用该工具前模型输出的过渡文本（同一轮内多个工具共享） -->
            <p
              v-if="call.preContent"
              class="mb-1 rounded bg-gray-50/80 px-2 py-1 text-[11px] leading-relaxed text-gray-500 whitespace-pre-wrap break-words"
            >
              {{ call.preContent }}
            </p>
            <button
              type="button"
              class="flex w-full items-center gap-1.5 rounded px-1 py-0.5 text-left transition-colors hover:bg-gray-100"
              @click="toggleDetail(i)"
            >
              <span class="truncate text-xs font-medium text-gray-600">{{ call.name }}</span>
              <span class="shrink-0 text-[11px]" :class="call.status === 'done' ? 'text-green-500' : 'text-primary-500'">
                {{ call.status === 'done' ? '完成' : '执行中' }}
              </span>
              <CheckCircleIcon v-if="call.status === 'done'" class="h-3 w-3 shrink-0 text-green-500" />
              <ChevronDownIcon class="h-3 w-3 shrink-0 text-gray-400 transition-transform duration-200" :class="openIdx === i ? 'rotate-180' : ''" />
            </button>
            <Transition name="detail">
              <div v-if="openIdx === i" class="space-y-1.5 py-1.5 pl-1 text-[11px]">
                <div>
                  <div class="mb-0.5 flex items-center justify-between text-gray-400">
                    <span>入参（arguments）</span>
                    <button type="button" class="rounded px-1 text-gray-400 transition-colors hover:text-primary-500" @click="copy(call.arguments)">
                      复制
                    </button>
                  </div>
                  <pre class="max-h-32 overflow-auto rounded bg-gray-50 p-1.5 font-mono whitespace-pre-wrap break-all text-gray-600">{{ call.arguments || '（无）' }}</pre>
                </div>
                <div v-if="call.status === 'done'">
                  <div class="mb-0.5 flex items-center justify-between text-gray-400">
                    <span>执行结果（output）</span>
                    <button type="button" class="rounded px-1 text-gray-400 transition-colors hover:text-primary-500" @click="copy(call.output)">
                      复制
                    </button>
                  </div>
                  <pre class="max-h-40 overflow-auto rounded bg-gray-50 p-1.5 font-mono whitespace-pre-wrap break-all text-gray-600">{{ call.output || '（空）' }}</pre>
                </div>
              </div>
            </Transition>
          </div>
        </li>
      </TransitionGroup>
    </Transition>
  </div>
</template>

<style scoped>
/* 左侧虚线贯穿所有节点 */
.tool-timeline {
  list-style: none;
  margin: 0.25rem 0 0;
  padding: 0;
}

.tool-item {
  position: relative;
  padding-left: 0.75rem;
  padding-bottom: 0.25rem;
}

.tool-item::before {
  content: '';
  position: absolute;
  left: 0.125rem;
  top: 0.875rem;
  bottom: 0;
  border-left: 1px dashed #d0d5dd;
}

.tool-item:last-child::before {
  border-left-style: dashed;
}

.tool-dot {
  position: absolute;
  left: 0;
  top: 0.375rem;
  width: 0.3125rem;
  height: 0.3125rem;
  border-radius: 9999px;
  background-color: #ffffff;
  border: 1.5px solid #d0d5dd;
  box-sizing: border-box;
}

.tool-dot.is-running {
  border-color: var(--color-primary-500, #4f6ef2);
  background-color: var(--color-primary-500, #4f6ef2);
}

.tool-dot.is-done {
  border-color: #00b42a;
  background-color: #00b42a;
}

/* ===== 展开/收起动画 ===== */
/* 整条时间线：淡入 + 轻微下滑 */
.chain-enter-active,
.chain-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.chain-enter-from,
.chain-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* 工具节点逐个入场：淡入 + 轻微右移 */
.node-enter-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.node-enter-from {
  opacity: 0;
  transform: translateX(-6px);
}

.node-move {
  transition: transform 0.2s ease;
}

/* 节点详情展开：淡入 + 轻微下移 */
.detail-enter-active,
.detail-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.detail-enter-from,
.detail-leave-to {
  opacity: 0;
  transform: translateY(-3px);
}
</style>
