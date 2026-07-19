<script setup lang="ts">
/**
 * 皮肤与披风管理弹窗（参考 PCL2 的 PageLoginMsSkin）
 *
 * - 微软账号：3D 预览 + 上传皮肤 + 装备/取消披风
 * - 离线账号：3D 预览 + 本地默认皮肤选择（保存到注册表，按 uuid 绑定）
 *
 * 业务逻辑已抽取到 `@/composables/useSkinOperations`，本文件仅负责模板组装。
 *
 * 子组件（skin-manager/）：
 *   - SkinAnimationSelector  动画状态选择
 *   - SkinCapeList           披风列表（微软）
 *   - SkinUploadPanel        上传皮肤 + 账号管理（微软）
 *   - SkinLocalSelector      本地皮肤选择（离线）
 */

import { ref, computed, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import SkinAnimationSelector, { type AnimationType } from './skin-manager/SkinAnimationSelector.vue'
import SkinCapeList from './skin-manager/SkinCapeList.vue'
import SkinUploadPanel from './skin-manager/SkinUploadPanel.vue'
import SkinLocalSelector from './skin-manager/SkinLocalSelector.vue'
import SkinPreviewPanel from './skin-manager/SkinPreviewPanel.vue'
import { useSkinOperations } from '@/composables/useSkinOperations'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean] }>()

const authStore = useAuthStore()
const animation = ref<AnimationType>('idle')

const uuid = computed(() => authStore.currentUser?.uuid ?? '')
const username = computed(() => authStore.currentUser?.name ?? '')
const isMicrosoft = computed(() => authStore.currentUser?.login_type === 'Microsoft')

const {
  info, loading, uploading, skinUrl, capeUrl, variant, selectedLocalSkin,
  activeCape,
  loadInfo, pickAndUpload, onEquipCape, onUnequipCape,
  onSelectLocalSkin, saveSkinToLocal,
} = useSkinOperations({ uuid, username, isMicrosoft })

function close() {
  emit('update:visible', false)
}

watch(() => props.visible, (v) => {
  if (v) loadInfo()
})
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
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="close"
      >
        <div class="absolute inset-0 bg-black/40" />

        <div class="relative w-full max-w-2xl rounded-lg bg-white shadow-xl">
          <!-- 头部 -->
          <div class="flex items-center justify-between border-b border-gray-100 px-5 py-3">
            <h3 class="text-base font-semibold text-gray-900">
              {{ isMicrosoft ? '皮肤与披风管理' : '本地皮肤选择' }}
            </h3>
            <button
              class="rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600"
              @click="close"
            >
              <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M4.3 4.3a1 1 0 011.4 0L10 8.6l4.3-4.3a1 1 0 111.4 1.4L11.4 10l4.3 4.3a1 1 0 01-1.4 1.4L10 11.4l-4.3 4.3a1 1 0 01-1.4-1.4L8.6 10 4.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" /></svg>
            </button>
          </div>

          <!-- 内容 -->
          <div class="max-h-[70vh] overflow-y-auto p-5">
            <div v-if="loading" class="py-12 text-center text-sm text-gray-400">
              <svg class="mx-auto mb-2 h-6 w-6 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M12 2a10 10 0 0110 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
              </svg>
              加载中...
            </div>

            <div v-else class="grid grid-cols-1 gap-5 md:grid-cols-2">
              <!-- 左：3D 人物预览（微软和离线账号共用） -->
              <SkinPreviewPanel
                :skin-url="skinUrl"
                :cape-url="capeUrl"
                :variant="variant"
                :animation="animation"
                :uuid="uuid"
                :username="username"
                :is-microsoft="isMicrosoft"
                :active-cape="activeCape"
                :selected-local-skin="selectedLocalSkin"
                @save="saveSkinToLocal"
              />

              <!-- 右：上传（微软）/ 皮肤选择（离线） + 动画 -->
              <div class="space-y-4">
                <!-- 微软账号：上传皮肤 + 账号管理 -->
                <SkinUploadPanel
                  v-if="isMicrosoft"
                  v-model:variant="variant"
                  :uploading="uploading"
                  @upload="pickAndUpload"
                />

                <!-- 离线账号：本地皮肤选择网格 -->
                <SkinLocalSelector
                  v-else
                  :selected-local-skin="selectedLocalSkin"
                  @select="onSelectLocalSkin"
                />

                <!-- 动画状态（所有账号类型共用） -->
                <SkinAnimationSelector v-model="animation" />
              </div>

              <!-- 披风列表（仅微软账号，横跨两列） -->
              <SkinCapeList
                v-if="isMicrosoft && info"
                :capes="info.capes"
                :active-cape="activeCape"
                :uploading="uploading"
                @equip="onEquipCape"
                @unequip="onUnequipCape"
              />
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
