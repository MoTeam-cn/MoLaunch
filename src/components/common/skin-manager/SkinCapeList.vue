<script setup lang="ts">
/**
 * 披风列表（仅微软账号）
 *
 * - 展示所有可用披风，点击装备
 * - 当前已装备披风高亮 + "取消当前披风"按钮
 * - emit equip/unequip，业务逻辑由父组件处理
 */
import type { SkinCapeInfo } from '@/utils/tauri'

defineProps<{
  capes: SkinCapeInfo['capes']
  activeCape: SkinCapeInfo['capes'][number] | null
  uploading: boolean
}>()

const emit = defineEmits<{
  equip: [capeId: string]
  unequip: []
}>()
</script>

<template>
  <div class="rounded-lg border border-gray-100 p-4 md:col-span-2">
    <div class="mb-3 flex items-center justify-between">
      <div class="text-sm font-medium text-gray-700">披风列表</div>
      <button
        v-if="activeCape"
        class="rounded-md border border-red-200 px-2 py-1 text-xs text-red-500 transition-colors hover:bg-red-50 disabled:opacity-50"
        :disabled="uploading"
        @click="emit('unequip')"
      >取消当前披风</button>
    </div>
    <div v-if="capes.length > 0" class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
      <button
        v-for="cape in capes"
        :key="cape.id"
        class="flex items-center gap-2 rounded-md border px-2 py-2 text-left text-xs transition-colors disabled:opacity-50"
        :class="cape.state === 'ACTIVE' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-700 hover:bg-gray-50'"
        :disabled="uploading || cape.state === 'ACTIVE'"
        @click="emit('equip', cape.id)"
      >
        <svg class="h-4 w-4 flex-none" viewBox="0 0 20 20" fill="currentColor">
          <path v-if="cape.state === 'ACTIVE'" fill-rule="evenodd" d="M16.7 5.3a1 1 0 010 1.4l-8 8a1 1 0 01-1.4 0l-4-4a1 1 0 011.4-1.4L8 12.6l7.3-7.3a1 1 0 011.4 0z" clip-rule="evenodd" />
          <path v-else d="M3 5a2 2 0 012-2h10a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm2 1a1 1 0 011-1h8a1 1 0 110 2H6a1 1 0 01-1-1z" />
        </svg>
        <span class="flex-1 truncate">{{ cape.display_name }}</span>
      </button>
    </div>
    <div v-else class="py-6 text-center text-xs text-gray-400">暂无披风</div>
  </div>
</template>
