<script setup lang="ts">
/**
 * 自定义背景设置区
 *
 * 支持本地图片或 http(s) 图片地址、模糊半径、遮罩不透明度与填充方式；
 * 所有改动即时生效（写 localStorage + CSS 变量）。
 */
import { defineAsyncComponent } from 'vue'
import { useUiPrefsStore } from '@/stores/ui-prefs'
import { pickFile } from '@/utils/fileDialog'
import { toastInfo } from '@/utils/toast'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Slider = defineAsyncComponent(() => import('@/components/common/Slider.vue'))

const uiStore = useUiPrefsStore()

const fillOptions = [
  { label: '铺满裁剪', value: 'cover' },
  { label: '完整显示', value: 'contain' },
  { label: '平铺', value: 'repeat' },
]

/** 选择本地图片（路径直接写回配置） */
async function chooseImage() {
  const picked = await pickFile({
    title: '选择背景图片',
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!picked) return
  uiStore.setBackground({ image: picked })
  toastInfo('背景图已更新')
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">背景</h3>
    <div class="divide-y divide-gray-200">
      <!-- 图片地址 -->
      <div class="px-5 py-4 space-y-2">
        <p class="text-sm font-medium text-gray-900">自定义背景图</p>
        <p class="text-xs text-gray-500">支持本地图片路径（如 D:\\Pictures\\bg.png）或 http(s) 地址，留空则使用纯色背景</p>
        <div class="flex items-center gap-2">
          <Input
            :model-value="uiStore.background.image"
            placeholder="图片路径或 URL"
            class="flex-1"
            @update:model-value="uiStore.setBackground({ image: String($event) })"
          />
          <Button type="outline" size="small" @click="chooseImage">选择文件</Button>
          <Button
            v-if="uiStore.hasBackground"
            type="ghost"
            size="small"
            @click="uiStore.clearBackground()"
          >
            清除
          </Button>
        </div>
      </div>
      <!-- 模糊 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">背景模糊</p>
            <p class="text-xs text-gray-500 mt-0.5">模糊越强，背景越柔和，文字也越易读</p>
          </div>
          <div class="flex-none w-40">
            <Slider
              :model-value="uiStore.background.blur"
              :min="0"
              :max="32"
              :step="1"
              @update:model-value="uiStore.setBackground({ blur: Number($event) })"
            />
          </div>
        </div>
      </div>
      <!-- 遮罩 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">背景遮罩</p>
            <p class="text-xs text-gray-500 mt-0.5">数值越大背景越淡、界面越清晰</p>
          </div>
          <div class="flex-none w-40">
            <Slider
              :model-value="uiStore.background.mask"
              :min="0"
              :max="100"
              :step="1"
              @update:model-value="uiStore.setBackground({ mask: Number($event) })"
            />
          </div>
        </div>
      </div>
      <!-- 填充方式 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">填充方式</p>
            <p class="text-xs text-gray-500 mt-0.5">铺满裁剪会保持比例填满窗口；平铺适合小尺寸纹理图</p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="uiStore.background.fill"
              :options="fillOptions"
              @update:model-value="uiStore.setBackgroundFill($event as 'cover' | 'contain' | 'repeat')"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
