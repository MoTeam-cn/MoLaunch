<script setup lang="ts">
/**
 * 皮肤与披风管理弹窗（三分流：微软 / 外置 / 离线）
 *
 * - 微软账号：3D 预览 + 上传皮肤 + 装备/取消披风（SkinCapeList）
 * - 外置账号（yggdrasil）：3D 预览 + 上传/删除皮肤 + 上传/删除披风
 *   - 根据 uploadableTextures 动态显示上传按钮（仅 skin / 仅 cape / 二者 / 无）
 *   - 不显示本地默认皮肤选择（外置账号走 yggdrasil API）
 * - 离线账号：3D 预览 + 本地默认皮肤选择（保存到注册表，按 uuid 绑定）
 *
 * 业务逻辑已抽取到 `@/composables/useSkinOperations`，本文件仅负责模板组装。
 *
 * 子组件（skin-manager/）：
 *   - SkinAnimationSelector  动画状态选择
 *   - SkinCapeList           披风列表（微软）
 *   - SkinUploadPanel        上传皮肤 + 账号管理（微软）
 *   - SkinLocalSelector      本地皮肤选择（离线）
 *   - SkinPreviewPanel       3D 预览
 */

import { ref, computed, watch, defineAsyncComponent } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useVersionStore } from '@/stores/version'
import SkinAnimationSelector, { type AnimationType } from './skin-manager/SkinAnimationSelector.vue'
const SkinCapeList = defineAsyncComponent(() => import('./skin-manager/SkinCapeList.vue'))
const SkinUploadPanel = defineAsyncComponent(() => import('./skin-manager/SkinUploadPanel.vue'))
const SkinLocalSelector = defineAsyncComponent(() => import('./skin-manager/SkinLocalSelector.vue'))
const SkinPreviewPanel = defineAsyncComponent(() => import('./skin-manager/SkinPreviewPanel.vue'))
const Alert = defineAsyncComponent(() => import('./Alert.vue'))
const Button = defineAsyncComponent(() => import('./Button.vue'))
const Tooltip = defineAsyncComponent(() => import('./Tooltip.vue'))
import { useSkinOperations } from '@/composables/useSkinOperations'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean] }>()

const authStore = useAuthStore()
const versionStore = useVersionStore()
const animation = ref<AnimationType>('idle')

const uuid = computed(() => authStore.currentUser?.uuid ?? '')
const username = computed(() => authStore.currentUser?.name ?? '')
const loginType = computed(() => authStore.currentUser?.login_type ?? '')
const serverUrl = computed(() => authStore.currentUser?.server_url ?? '')
const mcVersion = computed(() => versionStore.selectedVersion || '1.20.1')

const {
  isMicrosoft, isAuthlib, isOffline,
  info, loading, uploading, skinUrl, capeUrl, variant, selectedLocalSkin,
  authlibUsingDefaultSkin,
  activeCape, canUploadSkin, canUploadCape,
  loadInfo, pickAndUpload, onEquipCape, onUnequipCape,
  onSelectLocalSkin, onUploadCustomSkin, saveSkinToLocal,
  onDeleteAuthlibSkin, onUploadAuthlibCape, onDeleteAuthlibCape,
} = useSkinOperations({ uuid, username, loginType, serverUrl })

/** 外置账号无上传权限时显示提示 */
const authlibNoUpload = computed(() => isAuthlib.value && !canUploadSkin.value && !canUploadCape.value)

/** 弹窗标题（外置账号带服务器名） */
const dialogTitle = computed(() => {
  if (isMicrosoft.value) return '皮肤与披风管理'
  if (isAuthlib.value) return '外置账号皮肤管理'
  return '本地皮肤选择'
})

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
        class="modal-shell"
        @click.self="close"
      >
        <div class="absolute inset-0 bg-black/40" />

        <div class="modal-body max-w-2xl mt-2">
          <!-- 头部 -->
          <div class="flex items-center justify-between border-b border-gray-100 px-5 py-3">
            <h3 class="text-base font-semibold text-gray-900">
              {{ dialogTitle }}
            </h3>
            <Button
              type="ghost"
              size="mini"
              class="!h-7 !w-7 !p-0 text-gray-400 hover:!text-gray-600 hover:!bg-gray-100"
              @click="close"
            >
              <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M4.3 4.3a1 1 0 011.4 0L10 8.6l4.3-4.3a1 1 0 111.4 1.4L11.4 10l4.3 4.3a1 1 0 01-1.4 1.4L10 11.4l-4.3 4.3a1 1 0 01-1.4-1.4L8.6 10 4.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" /></svg>
            </Button>
          </div>

          <!-- 内容 -->
          <div class="modal-scroll p-5">
            <!-- 离线账号：顶部提示皮肤生效范围 -->
            <Alert
              v-if="isOffline"
              variant="soft"
              type="info"
              message="离线皮肤通过 UUID 调整 + 资源包替换实现，游戏内将显示选定皮肤。1.19.3+ 也会精确显示，但仅本地可见。"
              class="mb-4"
            />

            <!-- 外置账号：服务器不支持上传时提示 -->
            <Alert
              v-if="authlibNoUpload"
              variant="soft"
              type="info"
              message="此 yggdrasil 服务器不支持上传皮肤或披风（uploadableTextures 为空）。仅可查看当前角色材质。"
              class="mb-4"
            />

            <!-- 外置账号：未在皮肤站设置皮肤，已用 Steve 默认皮肤顶上 -->
            <Alert
              v-if="isAuthlib && authlibUsingDefaultSkin"
              variant="soft"
              type="info"
              message="当前账号未在皮肤站设置皮肤，已显示默认 Steve 皮肤。上传皮肤后将替换为此账号的形象。"
              class="mb-4"
            />

            <div v-if="loading" class="py-12 text-center text-sm text-gray-400">
              <svg class="mx-auto mb-2 h-6 w-6 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M12 2a10 10 0 0110 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
              </svg>
              加载中...
            </div>

            <div v-else class="grid grid-cols-1 gap-5 md:grid-cols-2">
              <!-- 左：3D 人物预览（所有账号类型共用） -->
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

              <!-- 右：上传（微软/外置）/ 皮肤选择（离线） + 动画 -->
              <div class="space-y-4">
                <!-- 微软账号：上传皮肤 + 账号管理 -->
                <SkinUploadPanel
                  v-if="isMicrosoft"
                  v-model:variant="variant"
                  :uploading="uploading"
                  @upload="pickAndUpload"
                />

                <!-- 外置账号：皮肤与披风上传/删除面板 -->
                <div v-else-if="isAuthlib" class="space-y-3">
                  <!-- 皮肤上传/删除（仅当服务器允许上传 skin） -->
                  <div v-if="canUploadSkin" class="rounded-lg border border-gray-100 p-4">
                    <div class="mb-3 text-sm font-medium text-gray-700">上传皮肤</div>
                    <div class="mb-3 text-xs text-gray-500">
                      支持 64x64 或 64x32 PNG<br/>
                      上传到 yggdrasil 服务器，所有玩家可见
                    </div>
                    <div class="mb-3">
                      <label class="mb-1 block text-xs text-gray-500">皮肤模型</label>
                      <div class="flex gap-2">
                        <Button
                          :type="variant === 'classic' ? 'primary' : 'outline'"
                          size="small"
                          class="flex-1"
                          @click="variant = 'classic'"
                        >Steve（经典）</Button>
                        <Button
                          :type="variant === 'slim' ? 'primary' : 'outline'"
                          size="small"
                          class="flex-1"
                          @click="variant = 'slim'"
                        >Alex（纤细）</Button>
                      </div>
                    </div>
                    <div class="flex gap-2">
                      <Button
                        type="primary"
                        class="flex-1"
                        :loading="uploading"
                        @click="pickAndUpload"
                      >
                        {{ uploading ? '处理中...' : '选择文件并上传' }}
                      </Button>
                      <Tooltip text="删除当前皮肤，恢复默认 Steve/Alex" position="top">
                        <Button
                          type="outline"
                          size="small"
                          :disabled="uploading || !skinUrl || authlibUsingDefaultSkin"
                          @click="onDeleteAuthlibSkin"
                        >删除</Button>
                      </Tooltip>
                    </div>
                  </div>

                  <!-- 披风上传/删除（仅当服务器允许上传 cape） -->
                  <div v-if="canUploadCape" class="rounded-lg border border-gray-100 p-4">
                    <div class="mb-3 text-sm font-medium text-gray-700">上传披风</div>
                    <div class="mb-3 text-xs text-gray-500">
                      PNG 文件，上传到 yggdrasil 服务器
                    </div>
                    <div class="flex gap-2">
                      <Button
                        type="primary"
                        class="flex-1"
                        :loading="uploading"
                        @click="onUploadAuthlibCape"
                      >
                        {{ uploading ? '处理中...' : '选择文件并上传' }}
                      </Button>
                      <Tooltip text="删除当前披风" position="top">
                        <Button
                          type="outline"
                          size="small"
                          :disabled="uploading || !capeUrl"
                          @click="onDeleteAuthlibCape"
                        >删除</Button>
                      </Tooltip>
                    </div>
                  </div>
                </div>

                <!-- 离线账号：本地皮肤选择网格 + 自定义上传 -->
                <SkinLocalSelector
                  v-else
                  :selected-local-skin="selectedLocalSkin"
                  :mc-version="mcVersion"
                  @select="onSelectLocalSkin"
                  @upload="onUploadCustomSkin"
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
