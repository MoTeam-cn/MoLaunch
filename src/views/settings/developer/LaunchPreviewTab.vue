<script setup lang="ts">
/**
 * 开发者 - 启动参数预览子页签
 *
 * 选择已安装版本，后端组装最终启动参数（JVM 参数 / classpath / 游戏参数）但不启动游戏。
 * 账号信息取当前登录账号；未登录时按离线兜底处理。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { listInstalledVersionsWithType } from '@/utils/api/version'
import { previewLaunchArgs, type LaunchArgsPreview } from '@/utils/api/launch'
import { useAuthStore } from '@/stores/auth'
import { toastError, toastSuccess } from '@/utils/toast'
import { safeCall } from '@/utils/async'
import { PaperAirplaneIcon, ClipboardDocumentIcon } from '@heroicons/vue/24/outline'

const authStore = useAuthStore()

const versions = ref<{ id: string; version_type: string }[]>([])
const selectedVersion = ref('')
const loading = ref(false)
const preview = ref<LaunchArgsPreview | null>(null)
const error = ref<string | null>(null)

const versionOptions = computed(() =>
  versions.value.map(v => ({ label: v.id, value: v.id })),
)

onMounted(async () => {
  const r = await safeCall(() => listInstalledVersionsWithType(), 'load installed versions')
  if (Array.isArray(r)) {
    versions.value = r
    if (r.length > 0) selectedVersion.value = r[0].id
  }
})

async function runPreview() {
  if (!selectedVersion.value) {
    toastError('请选择版本')
    return
  }
  loading.value = true
  preview.value = null
  error.value = null
  try {
    const user = authStore.currentUser
    preview.value = await previewLaunchArgs({
      versionId: selectedVersion.value,
      username: user?.name ?? '',
      uuid: user?.uuid ?? '',
      loginType: user?.login_type ?? 'Legacy',
    })
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function copyAll() {
  if (!preview.value) return
  const text = [
    `Java: ${preview.value.java_path}`,
    '',
    'JVM 参数:',
    ...preview.value.jvm_args.map(a => `  ${a}`),
    '',
    `主类: ${preview.value.main_class}`,
    '',
    'Classpath:',
    `  ${preview.value.classpath}`,
    '',
    '游戏参数:',
    ...preview.value.game_args.map(a => `  ${a}`),
  ].join('\n')
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    toastError('复制失败')
  }
}

/** 超长参数省略显示（仅展示前 20 字符，点击 Tag 复制完整值） */
function shortArg(arg: string): string {
  return arg.length > 20 ? arg.slice(0, 20) + '…' : arg
}

/** 点击 Tag 复制完整参数 */
async function copyArg(arg: string) {
  try {
    await navigator.clipboard.writeText(arg)
    toastSuccess('已复制参数')
  } catch {
    toastError('复制失败')
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3 flex items-center gap-2">
        <PaperAirplaneIcon class="w-4 h-4 text-gray-500" />
        启动参数预览
      </h3>

      <div class="mx-5 mb-4">
        <Alert
          type="info"
          :truncate="false"
          message="选择已安装版本，后端按当前配置组装最终启动参数（含 Java 检测、内存、隔离模式、认证信息），仅预览不启动游戏。token 已脱敏，不会返回 access_token。"
        />
      </div>

      <div class="px-5 pb-5 space-y-4">
        <div class="flex items-end gap-3">
          <div class="w-64">
            <p class="text-sm font-medium text-gray-900 mb-1.5">游戏版本</p>
            <Select v-model="selectedVersion" :options="versionOptions" placeholder="请选择版本" />
          </div>
          <Button type="primary" :loading="loading" @click="runPreview">
            <template #icon><PaperAirplaneIcon class="w-4 h-4" /></template>
            预览参数
          </Button>
          <Button v-if="preview" type="secondary" @click="copyAll">
            <template #icon><ClipboardDocumentIcon class="w-4 h-4" /></template>
            复制全部
          </Button>
        </div>

        <!-- 错误提示 -->
        <div v-if="error" class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-600 whitespace-pre-wrap break-all">
          {{ error }}
        </div>

        <!-- 预览结果 -->
        <div v-if="preview" class="rounded-md border border-gray-200 overflow-hidden">
          <div class="flex items-center justify-between px-3 py-2 bg-gray-50 border-b border-gray-200">
            <span class="text-xs font-semibold text-gray-700">预览结果 · {{ preview.version_id }}</span>
            <span class="text-xs text-gray-400 font-mono">{{ preview.login_type }} · {{ preview.username }}</span>
          </div>
          <div data-inner-scroll class="max-h-[20rem] overflow-y-auto divide-y divide-gray-100">
            <!-- Java -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-1.5">Java 路径</p>
              <p class="text-xs text-gray-700 font-mono break-all">{{ preview.java_path }}</p>
            </div>
            <!-- JVM 参数 -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-2">JVM 参数（{{ preview.jvm_args.length }}）</p>
              <div class="flex flex-wrap gap-1.5">
                <Tooltip v-for="(arg, i) in preview.jvm_args" :key="i" text="点击复制" position="top">
                  <Tag
                    size="small"
                    color="arcoblue"
                    class="max-w-full cursor-pointer"
                    @click="copyArg(arg)"
                  >{{ shortArg(arg) }}</Tag>
                </Tooltip>
              </div>
            </div>
            <!-- 主类 -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-1.5">主类</p>
              <p class="text-xs text-gray-700 font-mono break-all">{{ preview.main_class }}</p>
            </div>
            <!-- Classpath -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-1.5">Classpath</p>
              <p class="text-xs text-gray-700 font-mono whitespace-pre-wrap break-all">{{ preview.classpath }}</p>
            </div>
            <!-- 游戏参数 -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-2">游戏参数（{{ preview.game_args.length }}）</p>
              <div class="flex flex-wrap gap-1.5">
                <Tooltip v-for="(arg, i) in preview.game_args" :key="i" text="点击复制" position="top">
                  <Tag
                    size="small"
                    color="green"
                    class="max-w-full cursor-pointer"
                    @click="copyArg(arg)"
                  >{{ shortArg(arg) }}</Tag>
                </Tooltip>
              </div>
            </div>
            <!-- 目录信息 -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-1.5">目录信息</p>
              <div class="space-y-1 text-xs text-gray-700 font-mono break-all">
                <p><span class="font-semibold text-gray-500">游戏目录：</span>{{ preview.game_dir }}</p>
                <p><span class="font-semibold text-gray-500">资源目录：</span>{{ preview.assets_dir }}</p>
                <p><span class="font-semibold text-gray-500">资源索引：</span>{{ preview.asset_index }}</p>
              </div>
            </div>
            <!-- 账号信息 -->
            <div class="px-4 py-3">
              <p class="text-xs font-bold text-gray-900 mb-1.5">账号信息</p>
              <div class="space-y-1 text-xs text-gray-700 font-mono break-all">
                <p><span class="font-semibold text-gray-500">用户名：</span>{{ preview.username }}</p>
                <p><span class="font-semibold text-gray-500">UUID：</span>{{ preview.uuid }}</p>
                <p><span class="font-semibold text-gray-500">登录类型：</span>{{ preview.login_type }}</p>
                <p v-if="preview.server_url"><span class="font-semibold text-gray-500">外置服务器：</span>{{ preview.server_url }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>