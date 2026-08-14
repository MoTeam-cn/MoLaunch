<script setup lang="ts">
/**
 * 游戏版本选择弹窗
 *
 * 版本隔离开启后，日志 / Mods 等资源按版本存放，手动附加上下文前需先选择版本。
 * 版本列表来自后端 `list_installed_versions`。
 */
import { ref, watch, defineAsyncComponent } from 'vue'
import { XMarkIcon, CubeIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))

const props = defineProps<{
  visible: boolean
  versions: string[]
  title?: string
}>()

const emit = defineEmits<{
  select: [version: string]
  cancel: []
}>()

const keyword = ref('')

watch(
  () => props.visible,
  (v) => {
    if (v) keyword.value = ''
  },
)

const filtered = () => {
  const kw = keyword.value.trim().toLowerCase()
  if (!kw) return props.versions
  return props.versions.filter((v) => v.toLowerCase().includes(kw))
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[10000] flex items-center justify-center p-4"
        @click.self="emit('cancel')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-md bg-white rounded-lg shadow-xl">
          <div class="p-5">
            <div class="flex items-center justify-between">
              <h3 class="text-base font-semibold text-gray-900">{{ title || '选择游戏版本' }}</h3>
              <button class="text-gray-400 hover:text-gray-600" @click="emit('cancel')">
                <XMarkIcon class="w-5 h-5" />
              </button>
            </div>
            <div class="mt-3">
              <Input v-model="keyword" placeholder="搜索版本" size="small" />
            </div>
            <div class="mt-3 max-h-72 overflow-y-auto -mx-2 px-2">
              <div v-if="filtered().length === 0" class="flex flex-col items-center justify-center py-8 text-gray-400">
                <CubeIcon class="w-8 h-8 mb-2" />
                <span class="text-xs">未找到已安装的版本</span>
              </div>
              <div
                v-for="v in filtered()"
                :key="v"
                class="flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer hover:bg-gray-50 transition-colors"
                @click="emit('select', v)"
              >
                <CubeIcon class="w-4 h-4 text-gray-400 shrink-0" />
                <span class="flex-1 truncate text-sm text-gray-700">{{ v }}</span>
                <Button type="text" size="mini">选择</Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
