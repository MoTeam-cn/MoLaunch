<script setup lang="ts">
/**
 * 资源包编辑器 - 已打开包信息栏（图标/名称/格式/大小 + 保存操作）
 */
import { computed, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { formatBytes } from '@/utils/format'
import { ArrowDownTrayIcon, CubeIcon } from '@heroicons/vue/24/outline'
import type { RpOpenResult, RpTreeNode } from '@/utils/api/tools'

const props = defineProps<{
  current: RpOpenResult
  exporting: boolean
}>()

const emit = defineEmits<{
  'save-zip': []
  'save-as-zip': []
}>()

const fileCount = computed(() => countFiles(props.current.tree))

function countFiles(node?: RpTreeNode): number {
  if (!node) return 0
  return node.children.reduce(
    (sum, c) => sum + (c.kind === 'file' ? 1 : countFiles(c)),
    0,
  )
}
</script>

<template>
  <div class="flex items-center gap-3 px-5 py-3">
    <img
      v-if="current.icon_data_url"
      :src="current.icon_data_url"
      class="h-11 w-11 shrink-0 rounded border border-gray-300 object-contain"
      alt="包图标"
    />
    <div v-else class="grid h-11 w-11 shrink-0 place-items-center rounded border border-gray-300 bg-gray-50">
      <CubeIcon class="h-6 w-6 text-gray-400" />
    </div>
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span class="truncate font-medium text-gray-800">{{ current.name }}</span>
        <Tag :color="current.format === 'zip' ? 'blue' : 'green'">
          {{ current.format === 'zip' ? 'ZIP' : '文件夹' }}
        </Tag>
        <Tag v-if="current.pack_format != null">pack_format {{ current.pack_format }}</Tag>
        <Tag v-if="current.mc_version" color="purple">{{ current.mc_version }}</Tag>
      </div>
      <p class="mt-0.5 truncate text-xs text-gray-500">
        {{ formatBytes(current.size) }} · {{ fileCount }} 个文件
        <span v-if="current.description"> · {{ current.description }}</span>
      </p>
    </div>
    <div class="flex shrink-0 items-center gap-2">
      <Button
        v-if="current.is_zip && current.src_path"
        size="small"
        type="outline"
        :loading="exporting"
        :disabled="exporting"
        @click="emit('save-zip')"
      >
        <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
        保存 ZIP
      </Button>
      <Button
        size="small"
        :loading="exporting"
        :disabled="exporting"
        @click="emit('save-as-zip')"
      >
        <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
        另存为 ZIP
      </Button>
    </div>
  </div>
</template>