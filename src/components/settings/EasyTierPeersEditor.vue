<script setup lang="ts">
/**
 * 设置-联机 - easytier 公共节点编辑器（信令节点/中继节点均可填）
 *
 * 项目自建信令节点 wss://node1.molaunch.moiu.cn 默认内置且不在此展示（加载时过滤），
 * 用户仅管理自定义节点；运行时由后端兜底合并默认信令节点，保证组网必有可用节点。
 */
import { ref, watch, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { toastSuccess } from '@/utils/toast'
import { useConfigPage } from '@/composables/useConfigPage'
import { ServerStackIcon } from '@heroicons/vue/24/outline'

/** 项目自建信令节点（默认内置，前端隐藏；与后端 publics.rs 的 DEFAULT_SIGNALING_PEER 保持一致） */
const DEFAULT_SIGNALING_PEER = 'wss://node1.molaunch.moiu.cn'

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
    peersText.value = (cfg.onlineEasytierPublicPeers ?? []).filter((p) => p !== DEFAULT_SIGNALING_PEER).join('\n')
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
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">easytier 公共节点</h3>
    <div class="divide-y divide-gray-200">
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <div>
            <p class="text-sm font-medium text-gray-900">公共节点列表</p>
            <p class="text-xs text-gray-500 mt-0.5">公网组网用于穿越 NAT，信令节点与中继节点均可，每行一个节点（默认信令节点内置不显示）</p>
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
          placeholder="tcp://relay.example.com:11010&#10;wss://node.example.com"
          class="font-mono"
        />
        <div class="mt-2 flex items-center justify-between">
          <span class="text-xs text-gray-400">
            <template v-if="peersSaved">已保存（对新建的虚拟网络生效）</template>
            <template v-else>留空时仅使用内置默认信令节点</template>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>