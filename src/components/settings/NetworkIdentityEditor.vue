<script setup lang="ts">
/**
 * 设置-联机 - 设备名编辑器（房客侧 easytier hostname）
 *
 * 房主 hostname 由联机中心协议决定（`scaffolding-mc-server-{center_port}`，后端
 * discover_center 按此前缀识别），无法自定义；此处设备名仅作用于加入方节点标识，
 * 写入 `online.network_identity`，重新加入房间后生效。留空使用默认 `mo-launch-guest`。
 */
import { ref, watch, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { toastSuccess } from '@/utils/toast'
import { useConfigPage } from '@/composables/useConfigPage'
import { IdentificationIcon } from '@heroicons/vue/24/outline'

/** 设备名最大长度（easytier hostname 无硬限制，取合理上限避免误填） */
const MAX_NAME_LEN = 32

const nameText = ref('')
const nameSaved = ref(false)

const {
  loaded: loadedName,
  markDirty: markDirtyName,
  flushSave: flushSaveName,
} = useConfigPage({
  delay: 800,
  errorLabel: 'save network identity',
  onLoad: (cfg) => {
    nameText.value = cfg.onlineNetworkIdentity ?? ''
  },
})

watch(nameText, () => {
  nameSaved.value = false
  markDirtyName('onlineNetworkIdentity', nameText.value.trim())
})

async function handleSaveName() {
  await flushSaveName()
  nameSaved.value = true
  toastSuccess('已保存')
}
</script>

<template>
  <div v-if="!loadedName" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <div class="px-5 py-5">
      <div class="h-4 w-28 bg-gray-200 rounded animate-pulse mb-4" />
      <div class="h-8 bg-gray-100 rounded animate-pulse" />
    </div>
  </div>
  <div v-else class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">设备名</h3>
    <div class="divide-y divide-gray-200">
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <div>
            <p class="text-sm font-medium text-gray-900">虚拟网络内设备名</p>
            <p class="text-xs text-gray-500 mt-0.5">仅加入方生效，重新加入房间后展示；房主 hostname 由联机中心协议决定，不可修改</p>
          </div>
          <Button type="outline" size="small" @click="handleSaveName">
            <template #icon><IdentificationIcon class="w-4 h-4" /></template>
            保存
          </Button>
        </div>
        <Input
          v-model="nameText"
          :maxlength="MAX_NAME_LEN"
          clearable
          placeholder="mo-launch-guest"
        />
        <div class="mt-2 flex items-center justify-between">
          <span class="text-xs text-gray-400">
            <template v-if="nameSaved">已保存（重新加入房间后生效）</template>
            <template v-else>留空使用默认设备名</template>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
