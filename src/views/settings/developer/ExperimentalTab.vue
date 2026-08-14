<script setup lang="ts">
/**
 * 开发者 - 实验性功能子页签
 *
 * 包含 Modrinth CDN 直连开关与更新检测分支切换。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { getUpdateBranch, setUpdateBranch, type UpdateBranch } from '@/utils/api/developer'
import { getCurrentChannel, getVersionInfo } from '@/utils/version'
import { toastError, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))

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

// ==================== 更新检测分支 ====================
const versionInfo = getVersionInfo()

/** 与后端 channel_name 一致的版本后缀 → 更新分支推导 */
function derivedBranch(): string {
  const c = getCurrentChannel()
  if (c === 'rc' || c === 'beta') return 'beta'
  if (c === 'alpha' || c === 'canary') return 'alpha'
  return 'stable'
}

const branchLabels: Record<string, string> = {
  auto: '跟随版本',
  stable: 'Stable 正式版',
  beta: 'Beta 测试版',
  alpha: 'Alpha 内测版',
}

const branchOptions = Object.entries(branchLabels).map(([value, label]) => ({ label, value }))

const updateBranch = ref<UpdateBranch>('auto')

async function onUpdateBranchChange(v: string | number) {
  const branch = String(v) as UpdateBranch
  try {
    await setUpdateBranch(branch)
    updateBranch.value = branch
    toastInfo(
      branch === 'auto' ? '已恢复跟随版本推导更新分支' : `已切换到 ${branchLabels[branch]} 分支`,
    )
  } catch (e) {
    toastError('切换更新分支失败：' + e)
    updateBranch.value = await getUpdateBranch()
  }
}

onMounted(async () => {
  await safeCall(async () => {
    const config = await getConfigMap()
    modrinthCdnRawEnabled.value = config.modrinthCdnRawEnabled
    updateBranch.value = await getUpdateBranch()
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

      <!-- 更新检测分支 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">更新检测分支</p>
            <p class="text-xs text-gray-500 mt-0.5">
              切换检查更新时使用的发布分支（channel），不设置时跟随当前版本自动推导
            </p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="updateBranch"
              :options="branchOptions"
              @update:model-value="onUpdateBranchChange"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          当前版本 {{ versionInfo.raw }}（自动推导 {{ derivedBranch() }} 分支）；
          {{ updateBranch === 'auto' ? '更新检测分支跟随版本自动推导' : `已切换到 ${branchLabels[updateBranch]} 分支` }}
        </p>
      </div>
    </div>
  </div>
</template>
