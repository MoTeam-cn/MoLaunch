<script setup lang="ts">
/**
 * 结构 popup 浮窗组件（OL Overlay 内容）
 * 复用：Button.vue / format.ts / constants.ts / toast.ts
 */
import {
  ClipboardDocumentIcon, MapPinIcon, ArrowRightIcon, XMarkIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import { formatCoord, copyToClipboard } from '@/utils/seedmap/format'
import { getStructIcon, getStructIconUrl } from '@/utils/seedmap/constants'
import { toastSuccess, toastError } from '@/utils/toast'
import type { WorkerStructure } from '@/utils/seedmap/types'

const props = defineProps<{ struct: WorkerStructure }>()
const emit = defineEmits<{ goto: []; close: [] }>()

async function onCopy() {
  const ok = await copyToClipboard(formatCoord(props.struct.x, props.struct.z))
  if (ok) toastSuccess('已复制: ' + formatCoord(props.struct.x, props.struct.z))
  else toastError('复制失败')
}
</script>

<template>
  <div class="struct-popup">
    <div class="flex items-center gap-2 pb-2 border-b border-gray-100">
      <img
        v-if="getStructIconUrl(struct.stype)"
        :src="getStructIconUrl(struct.stype)"
        class="w-5 h-5"
        alt=""
      />
      <span
        v-else
        class="w-3 h-3 rounded-full inline-block"
        :style="{ backgroundColor: getStructIcon(struct.stype).color }"
      />
      <span class="font-medium text-sm text-gray-900">
        {{ getStructIcon(struct.stype).label }}
      </span>
      <Button type="ghost" size="mini" class="!w-6 !h-6 !p-0 ml-auto" @click="emit('close')">
        <XMarkIcon class="w-4 h-4 text-gray-500" />
      </Button>
    </div>
    <div class="py-2 text-xs text-gray-700 font-mono">
      {{ formatCoord(struct.x, struct.z) }}
    </div>
    <div class="pb-2 text-xs" :class="struct.viable ? 'text-green-600' : 'text-yellow-600'">
      {{ struct.viable ? '已通过群系校验' : '未通过群系校验' }}
    </div>
    <div class="flex gap-1">
      <Button type="secondary" size="mini" @click="onCopy">
        <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
        复制
      </Button>
      <Button type="primary" size="mini" @click="emit('goto')">
        <template #icon><MapPinIcon class="w-3.5 h-3.5" /></template>
        前往
        <ArrowRightIcon class="w-3 h-3 ml-1" />
      </Button>
    </div>
  </div>
</template>

<style scoped>
.struct-popup {
  background: white;
  border-radius: 6px;
  padding: 10px 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border: 1px solid #e5e7eb;
  min-width: 200px;
  pointer-events: auto;
}
</style>
