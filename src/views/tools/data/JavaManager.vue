<script setup lang="ts">
/**
 * Java 运行时列表
 *
 * 仅展示启动器检测到的 Java 运行时（路径/版本/大版本徽章），供参考。
 * Java 的切换（自动选择 / 指定某个 Java）在「设置 → 启动设置」中完成，
 * 此处不再承载选择交互，避免与设置页职责重复。
 *
 * - 重新检测按钮触发 store.refreshJava()
 * - AlertV2 提示用户前往设置页切换 Java
 */
import { onMounted } from 'vue'
import {
  CommandLineIcon,
  ArrowPathIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Tag from '@/components/common/Tag.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import { useJavaStore } from '@/stores/java'
import { toastSuccess, toastError } from '@/utils/toast'

const javaStore = useJavaStore()

onMounted(() => {
  if (!javaStore.javaLoaded) javaStore.detectJava()
})

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
      <h3 class="text-sm font-semibold text-gray-900">Java 运行时列表</h3>
      <span class="ml-auto text-xs text-gray-400">共 {{ javaStore.javaList.length }} 个</span>
      <Button type="outline" size="small" :loading="!javaStore.javaLoaded" @click="refresh">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        重新检测
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <AlertV2
        type="info"
        message="本页仅展示系统检测到的 Java 运行时。使用哪个 Java 请在「设置 → 启动设置」中切换（自动选择或指定），此处不做选择以免与设置页重复。"
      />

      <!-- Java 列表（限制高度，超出滚动） -->
      <div data-inner-scroll class="max-h-[320px] overflow-y-auto space-y-2">
        <div
          v-for="j in javaStore.javaList"
          :key="j.executable"
          class="flex items-center gap-3 rounded-lg border border-gray-200 bg-white px-3 py-2.5"
        >
          <CommandLineIcon class="h-4 w-4 flex-none text-gray-500" />
          <div class="flex-1 min-w-0">
            <Tooltip :text="j.executable" position="top" :delay="200" block>
              <div class="truncate text-sm font-medium text-gray-900">{{ j.executable }}</div>
            </Tooltip>
            <div class="text-xs text-gray-400">版本 {{ j.version }}</div>
          </div>
          <Tag size="small" color="blue" class="flex-none">Java {{ j.major_version }}</Tag>
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
    </div>
  </section>
</template>