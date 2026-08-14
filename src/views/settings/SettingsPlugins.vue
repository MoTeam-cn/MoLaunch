<script setup lang="ts">
/**
 * 设置 - 插件管理页面
 *
 * 布局结构：
 * 1. 顶部说明
 * 2. 插件系统运行逻辑（流程图步骤）→ PluginFlowSteps
 * 3. 已安装插件列表（含权限 tag 展示）→ PluginListSection
 * 4. 外部插件安装入口（文件夹 + ZIP + 示例导出）
 * 5. 可用权限说明表格 → PermissionTableSection
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { usePluginStore } from '@/stores/plugins'
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { toastInfo, toastSuccess, toastError } from '@/utils/toast'
import { exportPluginSample } from '@/utils/api/plugins'
import { pickFile, pickDirectory, pickSavePath } from '@/utils/fileDialog'
const PluginFlowSteps = defineAsyncComponent(() => import('./plugins/PluginFlowSteps.vue'))
const PluginListSection = defineAsyncComponent(() => import('./plugins/PluginListSection.vue'))
const PermissionTableSection = defineAsyncComponent(() => import('./plugins/PermissionTableSection.vue'))
import {
  FolderOpenIcon,
  DocumentArrowUpIcon,
  DocumentArrowDownIcon,
} from '@heroicons/vue/24/outline'

const pluginStore = usePluginStore()

/** 安装中状态（文件夹 / ZIP 独立，避免状态泄漏） */
const installingFolder = ref(false)
const installingZip = ref(false)
/** 任一安装进行中（用于禁用另一按钮，防止并发安装） */
const installingAny = computed(() => installingFolder.value || installingZip.value)

/** 从文件夹安装 */
async function onInstallFromFolder() {
  if (installingAny.value) return
  installingFolder.value = true
  try {
    const folder = await pickDirectory()
    if (!folder) {
      return
    }
    toastInfo('正在从文件夹安装插件...')
    const pluginId = await pluginStore.installFromDir(folder)
    toastSuccess(`插件安装成功：${pluginId}`)
  } catch (e) {
    toastError('安装插件失败：' + e)
  } finally {
    installingFolder.value = false
  }
}

/** 从 ZIP 安装 */
async function onInstallFromZip() {
  if (installingAny.value) return
  installingZip.value = true
  try {
    const zipPath = await pickFile({
      title: '选择插件 ZIP 文件',
      filters: [
        { name: 'ZIP 文件', extensions: ['zip'] },
      ],
    })
    if (!zipPath) {
      return
    }
    toastInfo('正在从 ZIP 安装插件...')
    const pluginId = await pluginStore.installFromZip(zipPath)
    toastSuccess(`插件安装成功：${pluginId}`)
  } catch (e) {
    toastError('安装插件失败：' + e)
  } finally {
    installingZip.value = false
  }
}

/** 导出示例插件模板（文件夹或 ZIP） */
async function onExportPluginSample(asZip: boolean) {
  try {
    if (asZip) {
      const savePath = await pickSavePath({
        title: '保存示例插件 ZIP',
        defaultPath: 'plugin-sample.zip',
        filters: [{ name: 'ZIP 文件', extensions: ['zip'] }],
      })
      if (!savePath) return
      await exportPluginSample(savePath, true)
      toastSuccess(`示例插件 ZIP 已导出至：${savePath}`)
    } else {
      const folder = await pickDirectory()
      if (!folder) return
      await exportPluginSample(folder, false)
      toastSuccess(`示例插件已导出至：${folder}`)
    }
  } catch (e) {
    toastError('导出示例失败：' + e)
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- 顶部说明 -->
    <Alert
      type="info"
      :truncate="false"
      message="插件可扩展启动器功能。内置插件随启动器编译，不受沙箱限制；外部插件从文件夹或 ZIP 安装，通过 iframe 沙箱隔离执行，仅能调用已声明权限的 SDK 方法。"
    />

    <!-- 插件系统运行逻辑 -->
    <PluginFlowSteps />

    <!-- 已安装插件列表 -->
    <PluginListSection />

    <!-- 外部插件安装入口 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外部插件</h3>
      <div class="px-5 py-6">
        <!-- 安装按钮 -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">安装插件</p>
            <p class="mt-0.5 text-xs text-gray-500">
              支持从本地文件夹或 ZIP 文件安装。ZIP 内可扁平包含 manifest.json，或包含一个根目录
            </p>
          </div>
          <div class="flex flex-none items-center gap-2">
            <Button type="outline" size="small" :disabled="installingAny" :loading="installingFolder" @click="onInstallFromFolder">
              <FolderOpenIcon class="mr-1 h-3.5 w-3.5" />
              {{ installingFolder ? '安装中...' : '文件夹' }}
            </Button>
            <Button type="primary" size="small" :disabled="installingAny" :loading="installingZip" @click="onInstallFromZip">
              <DocumentArrowUpIcon class="mr-1 h-3.5 w-3.5" />
              {{ installingZip ? '安装中...' : 'ZIP 文件' }}
            </Button>
          </div>
        </div>

        <!-- 导出示例插件模板（开发者测试用） -->
        <div class="mt-4 flex items-center justify-between rounded border border-dashed border-gray-300 bg-white/50 px-3 py-2">
          <div class="min-w-0">
            <p class="text-xs font-medium text-gray-700">导出插件示例模板</p>
            <p class="mt-0.5 text-[11px] text-gray-400">
              导出 manifest.json + index.html 示例文件供开发者测试调试
            </p>
          </div>
          <div class="flex flex-none items-center gap-2">
            <Button type="outline" size="small" @click="onExportPluginSample(false)">
              <FolderOpenIcon class="mr-1 h-3.5 w-3.5" />
              文件夹
            </Button>
            <Button type="outline" size="small" @click="onExportPluginSample(true)">
              <DocumentArrowDownIcon class="mr-1 h-3.5 w-3.5" />
              ZIP 文件
            </Button>
          </div>
        </div>

        <!-- manifest.json 示例（含 processPermissions） -->
        <div class="mt-4 rounded bg-gray-50 p-3 text-xs text-gray-500">
          <p class="mb-1 font-medium text-gray-700">manifest.json 格式示例：</p>
          <pre class="overflow-x-auto text-[11px] leading-relaxed">{
  "id": "my-plugin",
  "name": "我的插件",
  "description": "插件功能描述",
  "version": "1.0.0",
  "author": "作者名",
  "entry": "index.html",
  "permissions": ["getConfig", "listInstalledVersions", "getCacheStats"],
  "processPermissions": {
    "allowedCommands": ["java", "node", "python"],
    "timeoutMs": 30000,
    "maxConcurrent": 1
  },
  "windowPermissions": {
    "allowedDomains": ["example.com", "*.github.io"],
    "width": 800,
    "height": 600,
    "resizable": true
  }
}</pre>
          <p class="mt-1.5 text-[10px] text-gray-400">
            permissions 声明可调用的 SDK 方法；processPermissions 仅在 permissions 包含 "spawnProcess" 时生效；windowPermissions 仅在包含 "createWindow" 时生效
          </p>
        </div>
      </div>
    </div>

    <!-- 可用权限说明 -->
    <PermissionTableSection />
  </div>
</template>
