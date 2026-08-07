<template>
  <Teleport to="body">
    <Transition name="agreement-fade">
      <div v-if="visible" class="fixed inset-0 z-[10050] flex items-center justify-center p-4">
        <div data-tauri-drag-region class="absolute inset-0 bg-black/40" />

        <div class="relative w-full max-w-3xl overflow-hidden rounded-lg bg-white shadow-xl">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between gap-3 border-b border-gray-200 px-5 py-3.5">
            <div class="flex min-w-0 items-center gap-2">
              <ShieldCheckIcon class="h-5 w-5 shrink-0 text-primary-500" />
              <span class="truncate text-sm font-semibold text-gray-800">{{ config?.title || 'MoLaunch 用户协议' }}</span>
            </div>
            <span class="shrink-0 rounded bg-gray-100 px-1.5 py-0.5 text-[11px] text-gray-500">v{{ currentVersion }}</span>
          </div>

          <!-- 内容区 -->
          <div class="max-h-[min(56vh,26rem)] overflow-y-auto px-5 py-4">
            <!-- 加载中 -->
            <div v-if="loading" class="flex items-center justify-center py-14 text-[12px] text-gray-400">
              正在加载用户协议...
            </div>

            <!-- 加载失败 -->
            <div
              v-else-if="!config"
              class="flex flex-col items-center justify-center py-14 text-gray-400"
            >
              <DocumentTextIcon class="mb-2 h-8 w-8 text-gray-300" />
              <span class="text-[12px]">用户协议加载失败</span>
              <Button type="outline" size="small" class="mt-3" @click="showAgreement(currentVersion)">
                重试
              </Button>
            </div>

            <!-- 协议正文 -->
            <div v-else class="space-y-3 text-[12px] leading-relaxed text-gray-600">
              <p class="text-gray-700">{{ config.intro }}</p>
              <ul class="space-y-2">
                <li v-for="(section, i) in config.sections" :key="i" class="flex gap-2">
                  <span class="mt-[3px] h-1.5 w-1.5 shrink-0 rounded-full bg-primary-400" />
                  <span>{{ section }}</span>
                </li>
              </ul>

              <!-- 完整条款外链 + 使用前置说明（按钮靠左，文案靠右、同行不换行、不与文案垂直居中） -->
              <div class="flex items-end gap-2 border-t border-gray-100 pt-3">
                <div class="flex items-center gap-2">
                  <Button type="outline" size="small" @click="openDoc(config.termsUrl)">
                    <template #icon><ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" /></template>
                    {{ config.termsText }}
                  </Button>
                  <Button type="outline" size="small" @click="openDoc(config.privacyUrl)">
                    <template #icon><ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" /></template>
                    {{ config.privacyText }}
                  </Button>
                </div>
                <p class="ml-auto whitespace-nowrap text-[11px] leading-relaxed text-gray-400">{{ config.notice }}</p>
              </div>
            </div>
          </div>

          <!-- 底部：已读确认 / 取消挽留 + 操作按钮 -->
          <div class="border-t border-gray-100 bg-gray-50 px-5 py-3.5">
            <!-- 取消挽留态：二次确认是否退出 -->
            <div v-if="leaving" class="flex items-center justify-between gap-3">
              <p class="flex-1 text-[12px] leading-relaxed text-gray-600">确定要退出 MoLaunch 吗？</p>
              <div class="flex shrink-0 items-center gap-2">
                <Button type="ghost" size="small" @click="leaving = false">返回上一步</Button>
                <Button type="ghost" size="small" class="!text-red-500" @click="handleCancel">确定退出</Button>
              </div>
            </div>
            <!-- 正常态：已读确认 + 取消 / 同意 -->
            <div v-else class="flex items-center justify-between gap-3">
              <Checkbox v-model="agreed">
                <span class="text-[12px] text-gray-700">我已阅读并同意本《用户协议》</span>
              </Checkbox>
              <div class="flex shrink-0 items-center gap-2">
                <Button type="ghost" size="small" @click="leaving = true">取消</Button>
                <Button type="primary" size="small" :loading="loading" :disabled="!config || !agreed" @click="handleAgree">
                  {{ config?.confirmText || '同意并继续' }}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
/**
 * 《用户协议》全局门禁弹窗（首次启动必须同意后才能使用）
 *
 * - 强制弹窗：无关闭按钮 / 无遮罩点击关闭 / 无 Esc，仅「同意并继续」可解除
 * - 协议内容与完整条款外链来自 utils/userAgreement.ts（本地默认 + 远端下发预留）
 * - 同意后经 acceptUserAgreement 持久化到系统存储（userAgreed + 版本号）
 */
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  ShieldCheckIcon,
  DocumentTextIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import { openLink } from '@/utils/aboutLogos'
import { toastError, toastSuccess } from '@/utils/toast'
import {
  acceptUserAgreement,
  resolveUserAgreementConfig,
  type UserAgreementRemoteConfig,
} from '@/utils/userAgreement'

const visible = ref(false)
const loading = ref(false)
const currentVersion = ref(0)
const config = ref<UserAgreementRemoteConfig | null>(null)
/** 用户是否已勾选「已阅读并同意」（未勾选时禁用同意按钮） */
const agreed = ref(false)
/** 取消挽留态：点击「取消」后二次确认，避免误触直接退出 */
const leaving = ref(false)

/** 弹出弹窗：加载生效的协议配置并展示 */
async function showAgreement(version: number) {
  currentVersion.value = version
  agreed.value = false
  leaving.value = false
  visible.value = true
  loading.value = true
  try {
    config.value = await resolveUserAgreementConfig()
  } catch {
    config.value = null
  } finally {
    loading.value = false
  }
}

/** 同意并持久化：写入系统存储后关闭弹窗 */
async function handleAgree() {
  if (loading.value || !config.value || !agreed.value) return
  try {
    await acceptUserAgreement(currentVersion.value)
    visible.value = false
    toastSuccess('已同意《用户协议》，感谢使用 MoLaunch')
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
  }
}

/** 挽留确认后的最终退出：不经关闭行为分流，直接请求后端清理（frpc/TUN）后退出进程 */
function handleCancel() {
  invoke('request_exit').catch((e) => {
    console.error('[UserAgreement] request_exit failed:', e)
    toastError('退出失败，请稍后重试')
  })
}

/** 打开完整条款外链 */
function openDoc(url: string) {
  openLink(url)
}

defineExpose({ showAgreement })
</script>

<style scoped>
.agreement-fade-enter-active,
.agreement-fade-leave-active {
  transition: opacity 0.2s ease;
}

.agreement-fade-enter-from,
.agreement-fade-leave-to {
  opacity: 0;
}
</style>
