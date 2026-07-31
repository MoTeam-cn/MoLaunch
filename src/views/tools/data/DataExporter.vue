<script setup lang="ts">
/**
 * 启动器数据导出
 *
 * 勾选导出项（配置 / 版本列表 / 账号）→ 填写输出 zip 路径 → 调用
 * exportLauncherData → 展示导出结果。
 *
 * 账号信息会脱敏：微软账号仅保留用户名/UUID，离线账号不含 skin。
 * 输出路径默认预填为下载目录下的 molaunch-export.zip，用户可手动编辑。
 */
import { ref, onMounted, computed } from 'vue'
import {
  ArrowDownTrayIcon,
  ArrowUpTrayIcon,
  CheckCircleIcon,
  ShieldCheckIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Input from '@/components/common/Input.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { exportLauncherData, getDownloadDir } from '@/utils/api/tools'
import type { ExportResult } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'
import { pickSavePath } from '@/utils/fileDialog'

const includeConfig = ref(true)
const includeVersions = ref(true)
const includeAccounts = ref(false)
const outputPath = ref('')
const exporting = ref(false)
const result = ref<ExportResult | null>(null)

const canExport = computed(
  () => outputPath.value.trim() !== '' && (includeConfig.value || includeVersions.value || includeAccounts.value),
)

onMounted(async () => {
  try {
    const dir = await getDownloadDir()
    outputPath.value = `${dir}\\molaunch-export.zip`
  } catch (e) {
    outputPath.value = 'molaunch-export.zip'
  }
})

async function doExport() {
  if (!canExport.value) {
    toastError('请至少勾选一项导出内容并填写输出路径')
    return
  }
  exporting.value = true
  result.value = null
  try {
    const res = await exportLauncherData({
      output_path: outputPath.value.trim(),
      include_config: includeConfig.value,
      include_versions: includeVersions.value,
      include_accounts: includeAccounts.value,
    })
    result.value = res
    toastSuccess('导出成功')
  } catch (e) {
    toastError(`导出失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    exporting.value = false
  }
}

function itemLabel(key: string): string {
  switch (key) {
    case 'config': return '启动器配置'
    case 'versions': return '版本列表'
    case 'accounts': return '账号信息（已脱敏）'
    default: return key
  }
}

async function pickOutput() {
  const path = await pickSavePath({
    title: '选择导出 zip 保存位置',
    defaultPath: outputPath.value || 'molaunch-export.zip',
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (path) outputPath.value = path
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <ArrowUpTrayIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">启动器数据导出</h3>
    </div>
    <div class="px-5 pb-5 space-y-4">
      <p class="text-xs text-gray-500">
        将启动器的配置、已安装版本列表、账号信息打包为 zip 文件，便于备份或迁移。
      </p>

      <!-- 导出项勾选 -->
      <div class="space-y-2">
        <label class="block text-xs font-medium text-gray-700">选择导出内容</label>
        <div
          class="flex items-center gap-2 rounded-lg border border-gray-200 px-3 py-2 cursor-pointer hover:bg-gray-50"
          :class="{ 'border-primary-300 bg-primary-50/40': includeConfig }"
        >
          <Checkbox v-model="includeConfig" />
          <span class="text-sm text-gray-800">启动器配置</span>
          <span class="text-xs text-gray-400">（游戏目录、Java 路径、窗口设置等）</span>
        </div>
        <div
          class="flex items-center gap-2 rounded-lg border border-gray-200 px-3 py-2 cursor-pointer hover:bg-gray-50"
          :class="{ 'border-primary-300 bg-primary-50/40': includeVersions }"
        >
          <Checkbox v-model="includeVersions" />
          <span class="text-sm text-gray-800">版本列表</span>
          <span class="text-xs text-gray-400">（已安装版本 ID 与类型）</span>
        </div>
        <div
          class="flex items-center gap-2 rounded-lg border border-gray-200 px-3 py-2 cursor-pointer hover:bg-gray-50"
          :class="{ 'border-primary-300 bg-primary-50/40': includeAccounts }"
        >
          <Checkbox v-model="includeAccounts" />
          <span class="text-sm text-gray-800">账号信息</span>
          <span class="flex items-center gap-1 text-xs text-green-600">
            <ShieldCheckIcon class="h-3 w-3" />已脱敏（仅含用户名/UUID，不含 token）
          </span>
        </div>
      </div>

      <!-- 输出路径 -->
      <div>
        <label class="mb-1 block text-xs font-medium text-gray-700">输出 zip 路径</label>
        <Input v-model="outputPath" placeholder="导出 zip 的完整路径" clearable>
          <template #append>
            <FolderOpenIcon
              class="h-4 w-4 cursor-pointer text-gray-500 hover:text-primary-600 transition-colors"
              @click="pickOutput"
            />
          </template>
        </Input>
      </div>

      <!-- 导出按钮 -->
      <div class="flex justify-end">
        <Button type="primary" :loading="exporting" :disabled="!canExport" @click="doExport">
          <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
          {{ exporting ? '导出中...' : '导出' }}
        </Button>
      </div>

      <!-- 导出结果 -->
      <div v-if="result" class="rounded-lg bg-green-50 px-4 py-3">
        <div class="flex items-center gap-3">
          <CheckCircleIcon class="h-5 w-5 flex-none text-green-500" />
          <div class="flex-1">
            <span class="text-sm font-medium text-green-800">导出成功</span>
            <span class="ml-2 rounded-full bg-green-100 px-1.5 py-0.5 text-xs font-medium text-green-700">
              {{ formatBytes(result.file_size) }}
            </span>
          </div>
        </div>
        <div class="mt-1.5 pl-8 text-xs text-green-600">
          <div>路径：{{ result.file_path }}</div>
          <div class="mt-0.5">
            包含：{{ result.exported_items.map(itemLabel).join('、') }}
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
