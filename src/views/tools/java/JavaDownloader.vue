<script setup lang="ts">
/**
 * Java 一键下载器
 *
 * 预设档位为 Mojang 官方 Runtime 可下载的五个（Java 25/21/17/16/8）+「自定义」档——
 * 自定义作为快速选择里的一个选项，选中后才显示输入框（校验 1~2 位纯数字、8~26 区间）。
 * 下载源固定为 Mojang 官方 Java Runtime；下载目录强制固定为 `%APPDATA%\.minecraft\runtime\`
 * （与官方启动器一致），用 AlertV2 说明原因；非官方版本（9/11/15/18/20/22~24 等自定义版本）
 * 提示官方 Runtime 可能未提供，下载失败时后端会返回明确原因。
 *
 * - 版本校验复用 `isJavaMajorValid`，官方档判断复用 `hasOfficialRuntime`
 * - 下载复用 `JavaDownloadBar`（按 targetMajor 驱动）
 */
import { ref, computed, onMounted } from 'vue'
import { ArrowDownTrayIcon, CheckCircleIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Tag from '@/components/common/Tag.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import JavaDownloadBar from '@/views/version-settings/JavaDownloadBar.vue'
import { useJavaStore } from '@/stores/java'
import { toastSuccess } from '@/utils/toast'
import {
  isJavaMajorValid,
  hasOfficialRuntime,
  MIN_JAVA_MAJOR,
  MAX_JAVA_MAJOR,
} from '@/utils/api/java'

/** 预设档位：Mojang 官方 Runtime 可下载的五档（25/21/17/16/8，对齐官方 all.json） */
const PRESETS = [25, 21, 17, 16, 8]

const javaStore = useJavaStore()
const activePreset = ref<number>(21)
const customMode = ref(false)
const customText = ref('')

onMounted(() => {
  if (!javaStore.javaLoaded) javaStore.detectJava()
})

function selectPreset(major: number) {
  activePreset.value = major
  customMode.value = false
  customText.value = ''
}

function selectCustom() {
  customMode.value = true
  customText.value = ''
}

/** 自定义输入是否违规（自定义模式下有输入但格式/范围不合法） */
const customInvalid = computed(
  () => customMode.value && customText.value.trim() !== '' && !isJavaMajorValid(customText.value),
)

const customHint = computed(() => {
  if (customInvalid.value) {
    return `请输入 ${MIN_JAVA_MAJOR}~${MAX_JAVA_MAJOR} 之间的 1~2 位数字，不要包含特殊符号或小数点`
  }
  const major = effectiveMajor.value
  if (major && !hasOfficialRuntime(major)) {
    return '该版本官方 Runtime 可能未提供，下载失败属正常现象，建议使用预设档位'
  }
  return ''
})

/** 当前生效的 Java 大版本号（自定义模式下输入合法时取自定义值，否则取预设档） */
const effectiveMajor = computed<number | null>(() => {
  if (customMode.value) {
    return isJavaMajorValid(customText.value) ? Number(customText.value.trim()) : null
  }
  return activePreset.value
})

/** 该版本是否已在系统中安装 */
const installed = computed(() =>
  javaStore.javaList.some((j) => j.major_version === effectiveMajor.value),
)

async function onDownloaded() {
  await javaStore.listJava()
  toastSuccess(`Java ${effectiveMajor.value} 已安装，可直接使用`)
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <ArrowDownTrayIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">Java 下载器</h3>
      <span class="ml-auto text-xs text-gray-400">已安装 {{ javaStore.javaList.length }} 个 Java</span>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <AlertV2
        type="warning"
        message="下载目录强制固定为 %APPDATA%\.minecraft\runtime\（与官方启动器一致）。原因：该目录跨游戏目录共享、不受版本隔离影响，且不会随游戏目录删除而丢失，下载后启动器与官方启动器均可直接使用。"
      />
      <div class="text-xs text-gray-400">
        下载源：Mojang 官方 Java Runtime（piston-meta / piston-data）；若在「设置 → 下载」中配置了 BMCLAPI 镜像，将自动走镜像加速。预设仅提供官方可下载的 25 / 21 / 17 / 16 / 8。
      </div>

      <!-- 快速选择：预设档位 + 自定义 -->
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-xs text-gray-500 flex-none">快速选择：</span>
        <Button
          v-for="p in PRESETS"
          :key="p"
          :type="!customMode && activePreset === p ? 'primary' : 'outline'"
          size="small"
          @click="selectPreset(p)"
        >
          Java {{ p }}
        </Button>
        <Button :type="customMode ? 'primary' : 'outline'" size="small" @click="selectCustom">
          自定义
        </Button>
      </div>

      <!-- 自定义输入（仅自定义档选中时显示，宽度自适应不限制） -->
      <div v-if="customMode" class="flex items-center gap-2">
        <span class="text-xs text-gray-500 flex-none">自定义版本：</span>
        <div class="flex-1">
          <Input
            v-model="customText"
            placeholder="输入大版本号，如 17"
            :hint="customHint"
            :hint-type="customInvalid ? 'error' : 'default'"
          />
        </div>
      </div>

      <!-- 当前目标状态 + 下载（已安装时不显示下载按钮，下载按钮靠右） -->
      <div v-if="effectiveMajor" class="flex items-center gap-2">
        <span class="text-sm font-medium text-gray-900">目标版本：Java {{ effectiveMajor }}</span>
        <Tag v-if="installed" size="small" color="green" class="flex-none">
          <template #icon><CheckCircleIcon class="h-3 w-3" /></template>
          已安装
        </Tag>
        <Tag v-else-if="!hasOfficialRuntime(effectiveMajor)" size="small" color="orange" class="flex-none">
          官方可能未提供
        </Tag>
        <div v-if="!installed" class="ml-auto">
          <JavaDownloadBar :target-major="effectiveMajor" @downloaded="onDownloaded" />
        </div>
      </div>
    </div>
  </section>
</template>
