<script setup lang="ts">
/**
 * 开发者 - 证书与安全子页签
 *
 * 三部分：
 * 1. TLS 信任源模式（Select）：builtin / system / custom / 组合 / all
 * 2. 忽略 TLS 证书校验（Select，仅开发者模式生效）：开启后跳过所有证书校验
 * 3. 自定义证书管理（表格 + 添加/删除）：管理 certs 目录下的 .pem 文件
 *
 * 后端：
 * - trust_mode / ignore_tls 通过 applyConfig 持久化（AppConfig.tls + 注册表 IgnoreTls）
 * - 证书增删查通过 system_manager 的 list_custom_certs / add_custom_cert / remove_custom_cert
 */
import { ref, onMounted } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import {
  listCustomCerts,
  addCustomCert,
  removeCustomCert,
  type CustomCertInfo,
} from '@/utils/api/developer'
import { pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastError } from '@/utils/toast'
import { safeCall } from '@/utils/async'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import Alert from '@/components/common/Alert.vue'
import {
  PlusIcon,
  TrashIcon,
  ShieldCheckIcon,
} from '@heroicons/vue/24/outline'

// ==================== TLS 信任源模式 ====================
const tlsTrustMode = ref('builtin')

const trustModeOptions = [
  { label: '仅内置证书库', value: 'builtin' },
  { label: '仅系统证书库', value: 'system' },
  { label: '仅自定义证书', value: 'custom' },
  { label: '内置 + 自定义', value: 'builtin+custom' },
  { label: '系统 + 自定义', value: 'system+custom' },
  { label: '全部信任源', value: 'all' },
]

async function changeTrustMode(v: string | number) {
  const mode = String(v)
  try {
    await applyConfig({ tlsTrustMode: mode })
    tlsTrustMode.value = mode
  } catch (e) {
    toastError('设置信任源模式失败：' + e)
  }
}

// ==================== 忽略 TLS 证书校验 ====================
const ignoreTls = ref(false)

async function toggleIgnoreTls(v: boolean) {
  try {
    await applyConfig({ ignoreTls: v })
    ignoreTls.value = v
  } catch (e) {
    toastError('设置忽略 TLS 失败：' + e)
    ignoreTls.value = !v
  }
}

// ==================== 自定义证书管理 ====================
const certs = ref<CustomCertInfo[]>([])
const certsLoading = ref(false)
const addingCert = ref(false)

async function loadCerts() {
  certsLoading.value = true
  await safeCall(async () => {
    certs.value = await listCustomCerts()
  }, 'load custom certs')
  certsLoading.value = false
}

async function handleAddCert() {
  const path = await pickFile({
    title: '选择 PEM 证书文件',
    filters: [{ name: 'PEM 证书', extensions: ['pem', 'crt', 'cer'] }],
  })
  if (!path) return

  addingCert.value = true
  try {
    await addCustomCert(path)
    toastSuccess('证书添加成功')
    await loadCerts()
  } catch (e) {
    toastError('添加证书失败：' + e)
  } finally {
    addingCert.value = false
  }
}

async function handleRemoveCert(filename: string) {
  try {
    await removeCustomCert(filename)
    toastSuccess('证书已删除：' + filename)
    await loadCerts()
  } catch (e) {
    toastError('删除证书失败：' + e)
  }
}

// ==================== 初始化 ====================
onMounted(async () => {
  await Promise.all([
    safeCall(async () => {
      const config = await getConfigMap()
      tlsTrustMode.value = config.tlsTrustMode
      ignoreTls.value = config.ignoreTls
    }, 'load tls config'),
    loadCerts(),
  ])
})
</script>

<template>
  <div class="space-y-6">
    <!-- TLS 信任源模式 + 忽略 TLS -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">TLS 证书校验</h3>
      <div class="divide-y divide-gray-200">
        <!-- 信任源模式 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">信任源模式</p>
              <p class="text-xs text-gray-500 mt-0.5">
                控制 HTTPS 请求信任哪些根证书库（修改后立即重建 HTTP 客户端）
              </p>
            </div>
            <div class="flex-none w-48">
              <Select
                :model-value="tlsTrustMode"
                :options="trustModeOptions"
                @update:model-value="changeTrustMode"
              />
            </div>
          </div>
        </div>

        <!-- 忽略 TLS 证书校验 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">忽略 TLS 证书校验</p>
              <p class="text-xs text-gray-500 mt-0.5">
                跳过所有 HTTPS 证书验证（仅开发者模式生效，用于自签名证书调试）
              </p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="ignoreTls ? 'true' : 'false'"
                :options="[
                  { label: '已开启', value: 'true' },
                  { label: '已关闭', value: 'false' },
                ]"
                @update:model-value="toggleIgnoreTls($event === 'true')"
              />
            </div>
          </div>
          <div v-if="ignoreTls" class="mt-2">
            <Alert
              type="warning"
              :truncate="false"
              message="已开启忽略 TLS 证书校验：所有 HTTPS 请求将跳过证书验证，存在中间人攻击风险。请仅用于联机服务端自签名证书调试，调试完成后及时关闭。"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 自定义证书管理 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <div class="flex items-center justify-between px-5 pt-5 pb-3">
        <h3 class="text-sm font-semibold text-gray-900">自定义证书</h3>
        <Button
          type="primary"
          size="small"
          :loading="addingCert"
          @click="handleAddCert"
        >
          <template #icon><PlusIcon class="w-3.5 h-3.5" /></template>
          添加证书
        </Button>
      </div>
      <p class="text-xs text-gray-500 px-5 pb-3">
        证书存储在 %APPDATA%/.Molaunch/certs/ 目录，仅支持 .pem 格式。信任源模式包含「自定义」时生效。
      </p>

      <!-- 证书列表 -->
      <div v-if="certs.length > 0" class="border-t border-gray-200">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-gray-50 border-b border-gray-200">
              <th class="text-left font-medium text-gray-600 px-5 py-2.5">文件名</th>
              <th class="text-left font-medium text-gray-600 px-5 py-2.5">Subject</th>
              <th class="text-left font-medium text-gray-600 px-5 py-2.5">过期时间</th>
              <th class="text-right font-medium text-gray-600 px-5 py-2.5">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-100">
            <tr v-for="cert in certs" :key="cert.filename" class="hover:bg-gray-50">
              <td class="px-5 py-3 font-mono text-xs text-gray-900 break-all">{{ cert.filename }}</td>
              <td class="px-5 py-3 text-gray-700 break-all">{{ cert.subject }}</td>
              <td class="px-5 py-3 text-gray-500 font-mono text-xs">{{ cert.notAfter || '-' }}</td>
              <td class="px-5 py-3 text-right">
                <Button
                  type="ghost"
                  size="mini"
                  @click="handleRemoveCert(cert.filename)"
                >
                  <template #icon><TrashIcon class="w-3.5 h-3.5" /></template>
                  删除
                </Button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!certsLoading" class="border-t border-gray-200 py-12 flex flex-col items-center justify-center gap-2 text-gray-400">
        <ShieldCheckIcon class="w-8 h-8" />
        <p class="text-sm">暂无自定义证书</p>
        <p class="text-xs">点击右上角「添加证书」导入 .pem 文件</p>
      </div>
    </div>
  </div>
</template>
