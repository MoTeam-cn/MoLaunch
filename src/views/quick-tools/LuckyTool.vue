<script setup lang="ts">
/**
 * 今日人品子组件（由 QuickTools.vue 承载）
 *
 * 基于本机设备 ID 与当前日期通过确定性哈希算法生成 0-100 幸运值：
 * 同一设备同一天结果固定，跨天自动重置，纯前端计算不依赖后端。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
import { SparklesIcon, QuestionMarkCircleIcon } from '@heroicons/vue/24/outline'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { getTodayLuck } from '@/utils/lucky'
import type { TodayLuck } from '@/utils/lucky'
import { getDeviceId } from '@/utils/api/java'
import { toastError } from '@/utils/toast'
import { maskDeviceId } from '@/utils/format'

const luck = ref<TodayLuck | null>(null)
const loading = ref(true)
const deviceId = ref('')

/** 等级 → 展示样式（类名静态映射，避免 Tailwind purge 丢失动态拼接类） */
const LEVEL_STYLES: Record<string, { tag: string; bar: string; text: string }> = {
  欧皇: { tag: 'gold', bar: 'bg-amber-400', text: 'text-amber-500' },
  小欧: { tag: 'green', bar: 'bg-green-400', text: 'text-green-600' },
  普通人: { tag: 'gray', bar: 'bg-slate-400', text: 'text-slate-600' },
  非酋: { tag: 'orange', bar: 'bg-orange-400', text: 'text-orange-500' },
  大非酋: { tag: 'red', bar: 'bg-red-400', text: 'text-red-500' },
}

const levelStyle = computed(() => LEVEL_STYLES[luck.value?.level ?? ''] ?? LEVEL_STYLES['普通人'])

onMounted(async () => {
  try {
    deviceId.value = await getDeviceId()
    luck.value = getTodayLuck(deviceId.value)
  } catch (e) {
    toastError(`获取设备标识失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <SparklesIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">今日人品</h3>
      <Tooltip
        text="基于本机设备 ID 与当前日期计算，同一设备每天结果固定，次日 0 点自动更新。结果仅供娱乐。"
      >
        <QuestionMarkCircleIcon class="h-4 w-4 text-gray-400 hover:text-gray-600" />
      </Tooltip>
    </div>

    <div class="px-5 pb-5">
      <p class="text-xs text-gray-500">
        结合设备与日期算出的每日幸运值，看看今天的手气如何。
      </p>

      <div v-if="loading" class="py-8 text-center text-xs text-gray-400">计算中...</div>

      <div v-else-if="luck" class="mt-4 flex items-center gap-6">
        <div class="flex-none text-center">
          <span :class="['text-5xl font-bold leading-none', levelStyle.text]">{{ luck.value }}</span>
          <p class="mt-2 text-xs text-gray-400">幸运值</p>
        </div>

        <div class="flex-1 space-y-3">
          <div class="flex items-center gap-2">
            <Tag :color="levelStyle.tag" size="medium">{{ luck.level }}</Tag>
            <span class="text-sm text-gray-600">{{ luck.comment }}</span>
          </div>
          <div class="h-2 w-full overflow-hidden rounded-full bg-gray-100">
            <div
              class="h-full rounded-full"
              :class="levelStyle.bar"
              :style="{ width: `${luck.value}%` }"
            />
          </div>
        </div>
      </div>

      <div v-else class="mt-6 py-4 text-center">
        <p class="text-xs text-gray-400">暂时无法获取设备标识，无法计算幸运值</p>
      </div>

      <div v-if="deviceId" class="mt-4 border-t border-gray-100 pt-3 text-xs text-gray-400">
        设备 {{ maskDeviceId(deviceId) }} · 每日 0 点自动更新
      </div>
    </div>
  </section>
</template>
