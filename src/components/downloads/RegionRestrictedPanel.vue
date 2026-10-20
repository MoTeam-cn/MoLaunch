<script setup lang="ts">
/**
 * 中国大陆地区下载限制提示面板
 *
 * 命中地区限制时替换下载页内容区：说明限制原因并提供官方渠道入口。
 * 文案与后端 utils::region::RESTRICTED_MESSAGE 保持一致。
 */
import { defineAsyncComponent } from 'vue'
import { ExclamationTriangleIcon, ArrowTopRightOnSquareIcon } from '@heroicons/vue/24/outline'
import { openExternal } from '@/utils/openExternal'
import { toastError, toastInfo } from '@/utils/toast'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

/** 我的世界官方网站（中国版下载入口） */
const OFFICIAL_URL = 'https://mc.163.com/'

/** 在系统浏览器打开官方网站 */
async function openOfficialSite() {
  toastInfo('正在打开我的世界官方网站…')
  try {
    await openExternal(OFFICIAL_URL)
  } catch (e) {
    console.error('[Region] 打开官方网站失败', e)
    toastError('打开失败，请手动访问 mc.163.com')
  }
}
</script>

<template>
  <div class="flex h-full items-center justify-center bg-gray-50 p-6">
    <div class="max-w-md text-center">
      <ExclamationTriangleIcon class="mx-auto h-12 w-12 text-amber-400" />
      <h3 class="mt-4 text-base font-semibold text-gray-900">当前地区暂不支持下载</h3>
      <p class="mt-2 text-sm leading-relaxed text-gray-500">
        受最近某事件影响，MoLaunch 启动器暂不向中国用户提供 MC 下载功能，恕我无能为力... qwq...
      </p>
      <div class="mt-5 flex justify-center">
        <Button type="primary" @click="openOfficialSite">
          <template #icon><ArrowTopRightOnSquareIcon class="h-4 w-4" /></template>
          前往官方网站
        </Button>
      </div>
      <p class="mt-4 text-xs text-gray-400">已安装的版本不受影响，仍可正常启动与游玩。</p>
    </div>
  </div>
</template>
