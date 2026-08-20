<script setup lang="ts">
/**
 * 导入 NAT 分享抽屉（Scaffolding 收敛版，参考 LobbyJoinDialog）
 *
 * 粘贴朋友分享的 NAT 内容，解析成功后 emit('imported') 由父组件更新拓扑图；
 * 失败时抽屉保持打开、内联展示错误可重试；成功时收起抽屉，父组件在 @close 后卸载本组件。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { ArrowDownTrayIcon } from '@heroicons/vue/24/outline'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { parseNatShare, type NatShareData } from '@/utils/online/nat-share'

const emit = defineEmits<{
  imported: [data: NatShareData]
  close: []
}>()

const visible = ref(false)
const shareText = ref('')
const errorMsg = ref('')

onMounted(() => {
  visible.value = true
})

/** 取消/遮罩/ESC：先播完关闭动画，@close 由 Drawer 在动画结束后触发 */
function handleCancel() {
  visible.value = false
}

/** 导入：空内容/格式非法内联提示；解析成功收起抽屉并通知父组件 */
function handleConfirm() {
  const text = shareText.value.trim()
  if (!text) {
    errorMsg.value = '请粘贴朋友分享的 NAT 内容'
    return
  }
  const data = parseNatShare(text)
  if (!data) {
    errorMsg.value = '分享内容无效，请确认完整复制'
    return
  }
  errorMsg.value = ''
  emit('imported', data)
  visible.value = false
}

/** Drawer 关闭动画结束后通知父组件卸载（v-if 移除） */
function onClosed() {
  emit('close')
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="520"
    render-in-place
    popup-container="#app-content"
    @update:visible="handleCancel"
    @close="onClosed"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <ArrowDownTrayIcon class="h-4 w-4 text-primary-500" />
        <span>导入 NAT 分享</span>
      </div>
    </template>

    <p class="text-sm text-gray-600">
      粘贴朋友分享的 NAT 内容，朋友侧节点将加入拓扑图并判断双方联机可能性
    </p>

    <div class="mt-4">
      <label class="mb-1 block text-sm font-medium text-gray-700">分享内容</label>
      <Input
        v-model="shareText"
        textarea
        :rows="4"
        class="w-full"
        placeholder="MoLaunchNATv1|..."
      />
    </div>

    <div v-if="errorMsg" class="mt-3 rounded-lg bg-red-50 p-3">
      <p class="text-sm text-red-600">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="handleCancel">取消</Button>
        <Button type="primary" size="small" @click="handleConfirm">导入</Button>
      </div>
    </template>
  </Drawer>
</template>