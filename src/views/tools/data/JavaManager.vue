<script setup lang="ts">
/**
 * Java 版本管理
 *
 * 复用 `stores/java.ts` + `utils/api/java.ts`，列出系统检测到的 Java 运行时，
 * 允许用户切换默认 Java 或使用自动选择模式。
 *
 * - 列表项展示路径、版本号、大版本徽章、64 位 / 手动导入 标签
 * - 单击列表项设为默认；"自动选择"行表示由后端启动流水线按版本需求自动匹配
 * - 重新检测按钮触发 store.refreshJava()
 */
import { onMounted } from 'vue'
import {
  CommandLineIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { useJavaStore } from '@/stores/java'
import { toastSuccess, toastInfo, toastError } from '@/utils/toast'

const javaStore = useJavaStore()

onMounted(() => {
  if (!javaStore.javaLoaded) javaStore.detectJava()
})

function selectJava(path: string) {
  javaStore.setJavaPath(path)
  if (path === '') {
    toastInfo('已切换为自动选择模式')
  } else {
    toastSuccess('已设为默认 Java')
  }
}

async function refresh() {
  try {
    await javaStore.refreshJava()
    toastSuccess(`已检测到 ${javaStore.javaList.length} 个 Java 运行时`)
  } catch (e) {
    toastError(`检测失败: ${e instanceof Error ? e.message : String(e)}`)
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CommandLineIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">Java 版本管理</h3>
      <span class="ml-auto text-xs text-gray-400">共 {{ javaStore.javaList.length }} 个</span>
      <Button type="outline" size="small" :loading="!javaStore.javaLoaded" @click="refresh">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        重新检测
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-2">
      <p class="text-xs text-gray-500">
        管理启动游戏使用的 Java 运行时。选择"自动选择"将由启动器根据游戏版本自动匹配最合适的 Java。
      </p>

      <!-- 自动选择 -->
      <div
        class="flex items-center gap-3 rounded-lg border px-3 py-2.5 cursor-pointer transition-colors"
        :class="
          javaStore.javaPath === ''
            ? 'border-primary-400 bg-primary-50/60'
            : 'border-gray-200 bg-white hover:bg-gray-50'
        "
        @click="selectJava('')"
      >
        <span
          class="flex h-4 w-4 flex-none items-center justify-center rounded-full border-2 transition-colors"
          :class="javaStore.javaPath === '' ? 'border-primary-500' : 'border-gray-300'"
        >
          <span v-if="javaStore.javaPath === ''" class="h-2 w-2 rounded-full bg-primary-500" />
        </span>
        <CpuChipIcon class="h-4 w-4 flex-none text-primary-500" />
        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium text-gray-900">自动选择</div>
          <div class="text-xs text-gray-400">由启动器根据游戏版本需求自动匹配</div>
        </div>
      </div>

      <!-- Java 列表（限制高度，超出滚动） -->
      <div class="max-h-[320px] overflow-y-auto space-y-2">
        <div
          v-for="j in javaStore.javaList"
          :key="j.executable"
          class="flex items-center gap-3 rounded-lg border px-3 py-2.5 cursor-pointer transition-colors"
          :class="
            javaStore.javaPath === j.executable
              ? 'border-primary-400 bg-primary-50/60'
              : 'border-gray-200 bg-white hover:bg-gray-50'
          "
          @click="selectJava(j.executable)"
        >
          <span
            class="flex h-4 w-4 flex-none items-center justify-center rounded-full border-2 transition-colors"
            :class="javaStore.javaPath === j.executable ? 'border-primary-500' : 'border-gray-300'"
          >
            <span v-if="javaStore.javaPath === j.executable" class="h-2 w-2 rounded-full bg-primary-500" />
          </span>
          <CommandLineIcon class="h-4 w-4 flex-none text-gray-500" />
          <div class="flex-1 min-w-0">
            <Tooltip :text="j.executable" position="top" :delay="200" block>
              <div class="truncate text-sm font-medium text-gray-900">{{ j.executable }}</div>
            </Tooltip>
            <div class="text-xs text-gray-400">版本 {{ j.version }}</div>
          </div>
          <span class="flex-none rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700">
            Java {{ j.major_version }}
          </span>
        </div>

        <!-- 空状态 -->
        <div
          v-if="javaStore.javaLoaded && javaStore.javaList.length === 0"
          class="flex flex-col items-center justify-center py-8 text-gray-400"
        >
          <ExclamationTriangleIcon class="h-8 w-8 mb-2" />
          <span class="text-xs">未检测到任何 Java 运行时，请点击右上角"重新检测"</span>
        </div>
      </div>

      <!-- 当前选中提示 -->
      <div v-if="javaStore.javaPath" class="flex items-center gap-2 pt-1 text-xs text-green-600">
        <CheckCircleIcon class="h-4 w-4" />
        当前默认：{{ javaStore.javaPath }}
      </div>
    </div>
  </section>
</template>
