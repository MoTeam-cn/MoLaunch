<script setup lang="ts">
/**
 * 皮肤与披风管理弹窗（参考 PCL2 的 PageLoginMsSkin）
 *
 * - 微软账号：3D 预览 + 上传皮肤 + 装备/取消披风
 * - 离线账号：3D 预览 + 本地默认皮肤选择（保存到注册表，按 uuid 绑定）
 *
 * 子组件（skin-manager/）：
 *   - SkinAnimationSelector  动画状态选择
 *   - SkinCapeList           披风列表（微软）
 *   - SkinUploadPanel        上传皮肤 + 账号管理（微软）
 *   - SkinLocalSelector      本地皮肤选择（离线）
 */

import { ref, computed, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import {
  getSkinCapeInfo, downloadSkinPng, downloadCapePng, uploadSkin, equipCape, unequipCape,
  selectFile, saveFile, saveDataUrlToFile, type SkinCapeInfo,
} from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import SkinAvatar from './SkinAvatar.vue'
import SkinModel3D from './SkinModel3D.vue'
import Tooltip from './Tooltip.vue'
import SkinAnimationSelector, { type AnimationType } from './skin-manager/SkinAnimationSelector.vue'
import SkinCapeList from './skin-manager/SkinCapeList.vue'
import SkinUploadPanel from './skin-manager/SkinUploadPanel.vue'
import SkinLocalSelector from './skin-manager/SkinLocalSelector.vue'
import { defaultSkins, getDefaultSkinEntry, getLocalSkinName, setLocalSkinName, bumpSkinVersion } from '@/utils/default-skin'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean] }>()

const authStore = useAuthStore()
const info = ref<SkinCapeInfo | null>(null)
const loading = ref(false)
const uploading = ref(false)
const skinDataUrl = ref<string | null>(null)
const capeDataUrl = ref<string | null>(null)
const variant = ref<'classic' | 'slim'>('classic')
/** 离线账号当前选中的本地皮肤名称 */
const selectedLocalSkin = ref<string | null>(null)
const animation = ref<AnimationType>('idle')

const uuid = computed(() => authStore.currentUser?.uuid ?? '')
const username = computed(() => authStore.currentUser?.name ?? '')
const isMicrosoft = computed(() => authStore.currentUser?.login_type === 'Microsoft')

/** 当前已装备的披风 */
const activeCape = computed(() => info.value?.capes.find(c => c.state === 'ACTIVE') ?? null)
/** 当前已装备的皮肤 */
const activeSkin = computed(() => info.value?.skins.find(s => s.state === 'ACTIVE') ?? info.value?.skins[0] ?? null)

async function loadInfo() {
  const dev = import.meta.env.DEV
  dev && console.log('[SkinManager] loadInfo started, isMicrosoft:', isMicrosoft.value)
  loading.value = true
  skinDataUrl.value = null
  capeDataUrl.value = null

  if (!isMicrosoft.value) {
    // 离线账号：使用本地默认皮肤（从注册表同步的内存缓存）
    const entry = getDefaultSkinEntry(uuid.value || username.value)
    skinDataUrl.value = entry.url
    variant.value = entry.variant
    selectedLocalSkin.value = getLocalSkinName(uuid.value) || entry.name
    info.value = null
    loading.value = false
    dev && console.log('[SkinManager] offline account, using local skin:', entry.name)
    return
  }

  // 微软账号：从后端获取最新皮肤/披风信息（后端操作成功后会自动刷新 profile_json）
  try {
    info.value = await getSkinCapeInfo()
    dev && console.log('[SkinManager] getSkinCapeInfo ok:', info.value)
  } catch (e) {
    console.error('[SkinManager] getSkinCapeInfo failed:', e)
    showError(`获取皮肤信息失败: ${e}`)
  }
  try {
    skinDataUrl.value = await downloadSkinPng()
    dev && console.log('[SkinManager] downloadSkinPng ok, length:', skinDataUrl.value?.length)
  } catch (e) {
    console.error('[SkinManager] downloadSkinPng failed:', e)
  }
  try {
    capeDataUrl.value = await downloadCapePng()
    dev && console.log('[SkinManager] downloadCapePng ok:', capeDataUrl.value ? 'has cape' : 'no cape')
  } catch (e) {
    console.warn('[SkinManager] downloadCapePng failed:', e)
    capeDataUrl.value = null
  }
  variant.value = activeSkin.value?.variant === 'slim' ? 'slim' : 'classic'

  loading.value = false
  dev && console.log('[SkinManager] loadInfo done, skinDataUrl:', skinDataUrl.value ? 'has data' : 'null')
}

async function pickAndUpload() {
  try {
    const filePath = await selectFile('选择皮肤 PNG 文件', [{ name: 'PNG 图片', extensions: ['png'] }])
    if (!filePath) return
    await runWithRefresh('皮肤上传成功', () => uploadSkin(filePath, variant.value))
  } catch (e) {
    showError(String(e))
  }
}

async function onEquipCape(capeId: string) {
  await runWithRefresh('披风已装备', () => equipCape(capeId))
}

async function onUnequipCape() {
  await runWithRefresh('披风已取消', () => unequipCape())
}

/** 上传/装备/取消操作后的通用流程：执行 → 提示 → 重新加载 + 触发头像刷新 */
async function runWithRefresh(successMsg: string, fn: () => Promise<unknown>) {
  uploading.value = true
  try {
    await fn()
    showSuccess(successMsg)
    await loadInfo()
    bumpSkinVersion()
  } catch (e) {
    showError(String(e))
  } finally {
    uploading.value = false
  }
}

/** 离线账号：选择本地默认皮肤 */
async function onSelectLocalSkin(skinName: string) {
  await setLocalSkinName(uuid.value, skinName)
  selectedLocalSkin.value = skinName
  const entry = defaultSkins.find(s => s.name === skinName)
  if (entry) {
    skinDataUrl.value = entry.url
    variant.value = entry.variant
  }
  bumpSkinVersion()
  showSuccess(`已切换为 ${skinName} 皮肤`)
}

/** 下载当前皮肤 PNG 到本地（弹出保存对话框） */
async function saveSkinToLocal() {
  if (!skinDataUrl.value) {
    showError('当前无皮肤数据')
    return
  }
  const defaultName = `${username.value || 'skin'}_${variant.value === 'slim' ? 'alex' : 'steve'}.png`
  const savePath = await saveFile('保存皮肤', defaultName, [{ name: 'PNG 图片', extensions: ['png'] }])
  if (!savePath) return
  try {
    await saveDataUrlToFile(skinDataUrl.value, savePath)
    showSuccess(`皮肤已保存到：${savePath}`)
  } catch (e) {
    showError('保存失败：' + String(e))
  }
}

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
              <div class="rounded-lg border border-gray-100 bg-gray-50/50 p-4">
                <div class="mb-3 flex items-center justify-between">
                  <div class="text-sm font-medium text-gray-700">当前形象</div>
                  <div class="text-[10px] text-gray-400">拖动旋转</div>
                </div>
                <!-- 3D 人物模型（skinview3d 渲染，皮肤 + 披风） -->
                <div class="flex justify-center rounded-md bg-white p-2 shadow-sm">
                  <SkinModel3D
                    :skin-url="skinDataUrl"
                    :cape-url="capeDataUrl"
                    :variant="variant"
                    :height="280"
                    :animation="animation"
                  />
                </div>
                <div class="mt-3 flex items-center gap-3">
                  <SkinAvatar :skin-url="skinDataUrl" :uuid="uuid" :username="username" :size="40" :overlay="true" :login-type="isMicrosoft ? 'Microsoft' : 'Offline'" />
                  <div class="flex-1 space-y-1 text-xs text-gray-500">
                    <div>用户名：{{ username }}</div>
                    <div>皮肤模型：{{ variant === 'slim' ? 'Alex（纤细）' : 'Steve（经典）' }}</div>
                    <div v-if="isMicrosoft">当前披风：{{ activeCape?.display_name ?? '未装备' }}</div>
                    <div v-else>当前皮肤：{{ selectedLocalSkin ?? '默认' }}</div>
                  </div>
                  <!-- 下载当前皮肤按钮 -->
                  <Tooltip text="下载当前皮肤 PNG 到本地" position="top" :delay="0">
                    <button
                      class="flex-none flex h-7 w-7 items-center justify-center rounded border border-gray-200 text-gray-600 transition-colors hover:bg-gray-50 disabled:opacity-40"
                      :disabled="!skinDataUrl"
                      @click="saveSkinToLocal"
                    >
                      <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M10 3a1 1 0 011 1v6.586l2.293-2.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 111.414-1.414L9 10.586V4a1 1 0 011-1z" />
                        <path d="M3 14a1 1 0 011 1v1h12v-1a1 1 0 112 0v2a1 1 0 01-1 1H3a1 1 0 01-1-1v-2a1 1 0 011-1z" />
                      </svg>
                    </button>
                  </Tooltip>
                </div>
              </div>

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
