<script setup lang="ts">
/**
 * 离线账号本地默认皮肤选择网格
 *
 * - 展示所有可选默认皮肤（Steve/Alex 等）
 * - 点击选中，emit select(skinName)
 * - 业务逻辑（保存到注册表 + 刷新预览）由父组件处理
 */
import SkinAvatar from '../SkinAvatar.vue'
import { defaultSkins } from '@/utils/default-skin'

defineProps<{
  selectedLocalSkin: string | null
}>()

const emit = defineEmits<{
  select: [skinName: string]
}>()
</script>

<template>
  <div class="rounded-lg border border-gray-100 p-4">
    <div class="mb-3 text-sm font-medium text-gray-700">选择默认皮肤</div>
    <div class="mb-3 text-xs text-gray-500">
      离线账号仅支持本地显示，选择后启动器和头像将显示该皮肤。
    </div>
    <div class="grid grid-cols-3 gap-2">
      <button
        v-for="skin in defaultSkins"
        :key="skin.name"
        class="flex flex-col items-center rounded-md border p-2 transition-colors"
        :class="selectedLocalSkin === skin.name
          ? 'border-primary-500 bg-primary-50 text-primary-700'
          : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
        @click="emit('select', skin.name)"
      >
        <SkinAvatar
          :skin-url="skin.url"
          :size="48"
          :rounded="false"
          :overlay="true"
        />
        <span class="mt-1 text-[10px]">{{ skin.name }}</span>
      </button>
    </div>
  </div>
</template>
