<script setup lang="ts">
/**
 * 离线账号本地皮肤选择网格 + 自定义皮肤上传
 *
 * - 根据 MC 版本显示可选默认皮肤（1.19.3+ 显示 9 个，旧版只显示 Steve/Alex）
 * - 支持上传自定义 PNG 皮肤文件
 * - 点击选中，emit select(skinName)
 * - 业务逻辑（保存到注册表 + 刷新预览）由父组件处理
 */
import { computed, defineAsyncComponent } from 'vue'
const SkinAvatar = defineAsyncComponent(() => import('../SkinAvatar.vue'))
const Button = defineAsyncComponent(() => import('../Button.vue'))
import { getDefaultSkinsForVersion, isCustomSkin } from '@/utils/default-skin'

const props = defineProps<{
  selectedLocalSkin: string | null
  /** 当前选中实例的 MC 版本（用于过滤可选皮肤） */
  mcVersion?: string
}>()

const emit = defineEmits<{
  select: [skinName: string]
  upload: []
}>()

/** 根据版本过滤后的可选皮肤列表 */
const availableSkins = computed(() => {
  const ver = props.mcVersion || '1.20.1'
  return getDefaultSkinsForVersion(ver)
})

/** 当前是否选中了自定义皮肤 */
const isCustomSelected = computed(() => isCustomSkin(props.selectedLocalSkin))
</script>

<template>
  <div class="rounded-lg border border-gray-100 p-4">
    <div class="mb-3 text-sm font-medium text-gray-700">选择默认皮肤</div>
    <div class="grid grid-cols-3 gap-2">
      <!-- 保留原生 button：皮肤列表项（flex-col + border + 选中态），
           Button.vue 的 scoped size 类与布局不适合网格列表项 -->
      <button
        v-for="skin in availableSkins"
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

    <!-- 自定义皮肤上传区 -->
    <div class="mt-4 border-t border-gray-100 pt-4">
      <div class="mb-2 text-sm font-medium text-gray-700">自定义皮肤</div>
      <div class="mb-3 text-xs text-gray-500">
        支持 64x64 或 64x32 PNG 文件，本地使用无大小限制
      </div>
      <Button
        :type="isCustomSelected ? 'primary' : 'outline'"
        size="small"
        long
        @click="emit('upload')"
      >
        {{ isCustomSelected ? '更换自定义皮肤' : '选择 PNG 文件上传' }}
      </Button>
    </div>
  </div>
</template>
