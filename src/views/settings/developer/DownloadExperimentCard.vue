<script setup lang="ts">
/**
 * 下载实验开关卡片
 *
 * 三个下载实验开关：受限 CurseForge 文件地址推导、边缘节点优选、DoH 解析。
 * 默认值分别是：关、关、关。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastError, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

const cfDeriveUrl = ref(false)
const edgeSelect = ref(false)
const doh = ref(false)

async function onCfDeriveUrlChange(next: boolean) {
  try {
    await applyConfig({ cfDeriveUrl: next })
    cfDeriveUrl.value = next
    toastInfo(next ? '已开启受限 CurseForge 文件地址推导' : '已关闭受限 CurseForge 文件地址推导')
  } catch (e) {
    toastError('切换受限文件推导失败：' + e)
    const cfg = await getConfigMap(true)
    cfDeriveUrl.value = cfg.cfDeriveUrl
  }
}

async function onEdgeSelectChange(next: boolean) {
  try {
    await applyConfig({ edgeSelect: next })
    edgeSelect.value = next
    toastInfo(next ? '已开启边缘节点优选' : '已关闭边缘节点优选')
  } catch (e) {
    toastError('切换边缘节点优选失败：' + e)
    const cfg = await getConfigMap(true)
    edgeSelect.value = cfg.edgeSelect
  }
}

async function onDohChange(next: boolean) {
  try {
    await applyConfig({ doh: next })
    doh.value = next
    toastInfo(next ? '已开启 DoH 解析' : '已关闭 DoH 解析')
  } catch (e) {
    toastError('切换 DoH 解析失败：' + e)
    const cfg = await getConfigMap(true)
    doh.value = cfg.doh
  }
}

onMounted(async () => {
  await safeCall(
    async () => {
      const cfg = await getConfigMap()
      cfDeriveUrl.value = cfg.cfDeriveUrl
      edgeSelect.value = cfg.edgeSelect
      doh.value = cfg.doh
    },
    'load download experiment config',
    () => toastError('加载下载实验开关失败'),
  )
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载实验开关</h3>
    <div class="divide-y divide-gray-200">
      <!-- 受限 CurseForge 文件地址推导 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">自动推导受限 CurseForge 文件地址</p>
            <p class="text-xs text-gray-500 mt-0.5">
              作者禁止第三方分发时 CurseForge 不返回下载地址，开启后按 fileId 推导 CDN 地址直接下载
            </p>
          </div>
          <div class="flex-none">
            <Checkbox :model-value="cfDeriveUrl" @update:model-value="onCfDeriveUrlChange" />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          {{
            cfDeriveUrl
              ? '已开启：受限文件按 fileId 推导地址下载，安装整合包不再因缺文件中断'
              : '已关闭（默认）：受限文件不推导，安装时会列出缺失文件并提示手动下载'
          }}
        </p>
        <p class="text-xs text-gray-400 mt-1">
          推导依赖 CurseForge 未公开的 CDN 路径布局，可能与官方行为不一致；仅建议在明确需要时开启。
        </p>
      </div>
      <!-- 边缘节点优选 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">边缘节点优选</p>
            <p class="text-xs text-gray-500 mt-0.5">
              对同一个 CDN 域名解析出的多个边缘 IP 各测一小段速度，挑最快的那个使用
            </p>
          </div>
          <div class="flex-none">
            <Checkbox :model-value="edgeSelect" @update:model-value="onEdgeSelectChange" />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          {{
            edgeSelect
              ? '已开启：每次优选最多消耗 4 MiB 探测流量，每小时上限 64 MiB'
              : '已关闭（默认）：直接用 DNS 给出的地址，不额外产生探测流量'
          }}
        </p>
        <p class="text-xs text-gray-400 mt-1">
          仅对公开 CDN 生效，不走代理、未关闭证书校验时才工作；节点连续失败会自动作废并重新优选。
        </p>
      </div>
      <!-- DoH 解析 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">DoH 解析</p>
            <p class="text-xs text-gray-500 mt-0.5">
              下载域名优先用 DoH 接口解析，避免运营商 DNS 劫持；失败自动回落系统解析
            </p>
          </div>
          <div class="flex-none">
            <Checkbox :model-value="doh" @update:model-value="onDohChange" />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          {{
            doh
              ? '已开启：腾讯 → 阿里 → Cloudflare 依次尝试，连续失败会熔断 5 分钟'
              : '已关闭（默认）：只用系统解析'
          }}
        </p>
        <p class="text-xs text-gray-400 mt-1">
          域名会发给第三方解析服务商，介意隐私时请保持关闭。
        </p>
      </div>
    </div>
  </div>
</template>
