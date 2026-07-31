<script setup lang="ts">
/**
 * 开发者 - 实验性功能子页签
 *
 * 当前仅包含 Modrinth CDN 直连开关。
 * 后续新增的实验性功能可在此页签内追加。
 */
import { ref, onMounted } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastError, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
import Select from '@/components/common/Select.vue'

// ==================== Modrinth CDN 直连 ====================
const modrinthCdnRawEnabled = ref(false)

async function toggleModrinthCdnRaw(v: boolean) {
  try {
    await applyConfig({ modrinthCdnRawEnabled: v })
    modrinthCdnRawEnabled.value = v
    toastInfo(v ? '已开启 Modrinth CDN 直连' : '已关闭 Modrinth CDN 直连')
  } catch (e) {
    toastError('设置 Modrinth CDN 直连失败：' + e)
    modrinthCdnRawEnabled.value = !v
  }
}

onMounted(async () => {
  await safeCall(async () => {
    const config = await getConfigMap()
    modrinthCdnRawEnabled.value = config.modrinthCdnRawEnabled
  }, 'load developer config', () => toastError('加载开发者配置失败'))
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">实验性功能</h3>
    <div class="divide-y divide-gray-200">
      <!-- Modrinth CDN 直连开关 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">Modrinth CDN 直连</p>
            <p class="text-xs text-gray-500 mt-0.5">
              将 cdn.modrinth.com 替换为 cdn-raw.modrinth.com（绕过中国大陆 cdn-alt 跳转）
            </p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="modrinthCdnRawEnabled ? 'true' : 'false'"
              :options="[
                { label: '已开启', value: 'true' },
                { label: '已关闭', value: 'false' },
              ]"
              @update:model-value="toggleModrinthCdnRaw($event === 'true')"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          <template v-if="modrinthCdnRawEnabled">已开启：Modrinth 下载走 cdn-raw 直连</template>
          <template v-else>已关闭：Modrinth 下载走官方 CDN（可能跳转 cdn-alt）</template>
        </p>
      </div>
    </div>
  </div>
</template>
