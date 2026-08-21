<script setup lang="ts">
/**
 * 性能设置页
 * - GPU 硬件加速开关（ToggleRow，关闭后 WebView2 走软件渲染降低内存占用，需重启生效）
 */
import { ref, defineAsyncComponent } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
import { toastSuccess, toastWarning } from '@/utils/toast'
const ToggleRow = defineAsyncComponent(() => import('@/components/settings/ToggleRow.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))

const useGpuAcceleration = ref(true)
const releaseMemoryOnTray = ref(false)

const { loaded, markDirty } = useConfigPage({
  delay: 500,
  errorLabel: 'save performance settings',
  onLoad: (cfg) => {
    useGpuAcceleration.value = cfg.useGpuAcceleration
    releaseMemoryOnTray.value = cfg.releaseMemoryOnTray
  },
})

function saveGpuAcceleration(v: boolean) {
  if (!loaded.value) return
  markDirty('useGpuAcceleration', v)
  toastWarning('GPU 硬件加速设置将在重启后生效')
}

function saveReleaseMemoryOnTray(v: boolean) {
  if (!loaded.value) return
  markDirty('releaseMemoryOnTray', v)
  toastSuccess(v ? '已开启，下次关闭到托盘时释放内存' : '已关闭')
}
</script>

<template>
  <div class="space-y-6">
    <!-- 加载占位 -->
    <div v-if="!loaded" class="space-y-6">
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
    </div>

    <template v-else>
      <!-- 渲染加速 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">渲染加速</h3>
        <div class="divide-y divide-gray-200">
          <ToggleRow
            v-model="useGpuAcceleration"
            label="GPU 硬件加速"
            description="开启时界面渲染使用 GPU 硬件加速，动画更流畅，占用约 35 MB 额外内存；低配电脑可关闭以降低内存占用"
            :hover="false"
            @update:model-value="(v) => saveGpuAcceleration(v as boolean)"
          />
        </div>
        <div class="px-5 pb-4">
          <Alert
            type="info"
            message="关闭后 3D 皮肤预览与模型预览将退化为软件渲染，可能变卡；该设置需重启 MoLaunch 后生效。"
          />
        </div>
      </div>

      <!-- 托盘内存释放 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">托盘内存释放</h3>
        <div class="divide-y divide-gray-200">
          <ToggleRow
            v-model="releaseMemoryOnTray"
            label="关闭到托盘时释放内存"
            description="挂起 WebView2 释放渲染资源，降低内存占用约 50~70%；从托盘恢复时秒回且保留当前界面状态"
            :hover="false"
            @update:model-value="(v) => saveReleaseMemoryOnTray(v as boolean)"
          />
        </div>
        <div class="px-5 pb-4">
          <Alert
            type="info"
            message="开启后关闭主界面时将挂起 WebView2 进程（渲染/GPU 资源释放），但浏览器进程仍保留；托盘图标点击恢复时自动解挂，界面状态完整保留。"
          />
        </div>
      </div>
    </template>
  </div>
</template>
