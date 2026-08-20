<script setup lang="ts">
/**
 * 设置-联机 - easytier 公共中继节点编辑器
 */
import { ref, watch, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { toastSuccess } from '@/utils/toast'
import { useConfigPage } from '@/composables/useConfigPage'
import { ServerStackIcon } from '@heroicons/vue/24/outline'

const peersText = ref('')
const peersSaved = ref(false)

const {
  loaded: loadedPeers,
  markDirty: markDirtyPeers,
  flushSave: flushSavePeers,
} = useConfigPage({
  delay: 800,
  errorLabel: 'save easytier peers',
  onLoad: (cfg) => {
    peersText.value = (cfg.onlineEasytierPublicPeers ?? []).join('\n')
  },
})

watch(peersText, () => {
  peersSaved.value = false
  markDirtyPeers('onlineEasytierPublicPeers', parsePeers())
})

function parsePeers(): string[] {
  return peersText.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
}

async function handleSavePeers() {
  await flushSavePeers()
  peersSaved.value = true
  toastSuccess('已保存')
}
</script>

<template>
  <div v-if="!loadedPeers" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <div class="px-5 py-5">
      <div class="h-4 w-28 bg-gray-200 rounded animate-pulse mb-4" />
      <div class="h-20 bg-gray-100 rounded animate-pulse" />
    </div>
  </div>
  <div v-else class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">easytier 公共中继节点</h3>
    <div class="divide-y divide-gray-200">
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <div>
            <p class="text-sm font-medium text-gray-900">中继节点列表</p>
            <p class="text-xs text-gray-500 mt-0.5">公网组网时用于穿越 NAT，每行一个节点</p>
          </div>
          <Button type="outline" size="small" @click="handleSavePeers">
            <template #icon><ServerStackIcon class="w-4 h-4" /></template>
            保存
          </Button>
        </div>
        <Input
          v-model="peersText"
          textarea
          :rows="4"
          placeholder="tcp://relay.example.com:11010&#10;udp://relay.example.com:11010"
          class="font-mono"
        />
        <div class="mt-2 flex items-center justify-between">
          <span class="text-xs text-gray-400">
            <template v-if="peersSaved">已保存（对新建的虚拟网络生效）</template>
            <template v-else>留空则不指定 --peers 参数</template>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>