<script setup lang="ts">
/**
 * 皮肤与披风管理弹窗（参考 PCL2 的 PageLoginMsSkin）
 *
 * - 微软账号：3D 预览 + 上传皮肤 + 装备/取消披风
 * - 离线账号：3D 预览 + 本地默认皮肤选择（保存到注册表，按 uuid 绑定）
 */

import { ref, computed, watch } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { useAuthStore } from '@/stores/auth'
import {
  getSkinCapeInfo, downloadSkinPng, downloadCapePng, uploadSkin, equipCape, unequipCape,
  selectFile, saveFile, saveDataUrlToFile, type SkinCapeInfo,
} from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import SkinAvatar from './SkinAvatar.vue'
import SkinModel3D from './SkinModel3D.vue'
import Tooltip from './Tooltip.vue'
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
/** 3D 预览动画类型 */
type AnimationType = 'idle' | 'walk' | 'run' | 'fly' | 'wave' | 'crouch' | 'hit' | 'swim' | 'none'
const animation = ref<AnimationType>('idle')
/** 动画选项：图标用 SVG path 数组（24x24 viewBox，stroke 风格） */
const animationOptions: { value: AnimationType; label: string; paths: string[] }[] = [
  { value: 'idle', label: '站立', paths: ['M12 2a2 2 0 100 4 2 2 0 000-4z', 'M12 6v10', 'M9 9l3-3 3 3', 'M9 20l3-4 3 4'] },
  { value: 'walk', label: '行走', paths: ['M13 4a2 2 0 100 4 2 2 0 000-4z', 'M13 8v6', 'M13 11l-3 2', 'M13 11l3 2', 'M13 14l-2 5', 'M13 14l2 5'] },
  { value: 'run', label: '跑步', paths: ['M14 3a2 2 0 100 4 2 2 0 000-4z', 'M14 7v6', 'M14 9l-4 1', 'M14 9l4 1', 'M14 13l-3 6', 'M14 13l3 6', 'M5 20h4'] },
  { value: 'fly', label: '飞行', paths: ['M12 2a2 2 0 100 4 2 2 0 000-4z', 'M12 6v8', 'M4 10l8-2 8 2', 'M9 20l3-6 3 6'] },
  { value: 'wave', label: '挥手', paths: ['M12 5a2 2 0 100 4 2 2 0 000-4z', 'M12 9v8', 'M12 11l-4-2', 'M12 12l4-2', 'M9 19l3-2 3 2'] },
  { value: 'crouch', label: '蹲下', paths: ['M12 4a2 2 0 100 4 2 2 0 000-4z', 'M12 8v5', 'M9 13h6', 'M8 13v5', 'M16 13v5'] },
  { value: 'hit', label: '受击', paths: ['M12 3a2 2 0 100 4 2 2 0 000-4z', 'M12 7v8', 'M9 10l-3-1', 'M15 10l3-1', 'M9 20l3-5 3 5'] },
  { value: 'swim', label: '游泳', paths: ['M5 8a2 2 0 100 4 2 2 0 000-4z', 'M3 14h4l4-2 4 2 4-1 2 1', 'M3 18h4l4-1 4 1 4-1 2 1'] },
  { value: 'none', label: '静止', paths: ['M12 3a2 2 0 100 4 2 2 0 000-4z', 'M12 7v10', 'M12 7l-3 3', 'M12 7l3 3', 'M9 21h6'] },
]

const uuid = computed(() => authStore.currentUser?.uuid ?? '')
const username = computed(() => authStore.currentUser?.name ?? '')
const isMicrosoft = computed(() => authStore.currentUser?.login_type === 'Microsoft')

/** 当前已装备的披风 */
const activeCape = computed(() => info.value?.capes.find(c => c.state === 'ACTIVE') ?? null)
/** 当前已装备的皮肤 */
const activeSkin = computed(() => info.value?.skins.find(s => s.state === 'ACTIVE') ?? info.value?.skins[0] ?? null)

async function loadInfo() {
  if (import.meta.env.DEV) console.log('[SkinManager] loadInfo started, isMicrosoft:', isMicrosoft.value)
  loading.value = true
  skinDataUrl.value = null
  capeDataUrl.value = null

  if (!isMicrosoft.value) {
    // 离线账号：使用本地默认皮肤（从注册表同步的内存缓存）
    const entry = getDefaultSkinEntry(uuid.value || username.value)
    skinDataUrl.value = entry.url
    capeDataUrl.value = null
    variant.value = entry.variant
    selectedLocalSkin.value = getLocalSkinName(uuid.value) || entry.name
    info.value = null
    loading.value = false
    if (import.meta.env.DEV) console.log('[SkinManager] offline account, using local skin:', entry.name)
    return
  }

  // 微软账号：从后端获取最新皮肤/披风信息
  // （后端 upload_skin/equip_cape/unequip_cape 成功后会自动刷新 profile_json，
  //   所以此处读取的是最新数据）

  // 1. 获取皮肤/披风信息
  try {
    info.value = await getSkinCapeInfo()
    if (import.meta.env.DEV) console.log('[SkinManager] getSkinCapeInfo ok:', info.value)
  } catch (e) {
    console.error('[SkinManager] getSkinCapeInfo failed:', e)
    showError(`获取皮肤信息失败: ${e}`)
  }

  // 2. 下载皮肤 PNG 全图用于 3D 预览
  try {
    skinDataUrl.value = await downloadSkinPng()
    if (import.meta.env.DEV) console.log('[SkinManager] downloadSkinPng ok, length:', skinDataUrl.value?.length)
  } catch (e) {
    console.error('[SkinManager] downloadSkinPng failed:', e)
  }

  // 3. 下载披风 PNG（可能为 null）
  try {
    capeDataUrl.value = await downloadCapePng()
    if (import.meta.env.DEV) console.log('[SkinManager] downloadCapePng ok:', capeDataUrl.value ? 'has cape' : 'no cape')
  } catch (e) {
    console.warn('[SkinManager] downloadCapePng failed:', e)
    capeDataUrl.value = null
  }

  // 4. 同步 variant
  if (activeSkin.value?.variant === 'slim') {
    variant.value = 'slim'
  } else {
    variant.value = 'classic'
  }

  loading.value = false
  if (import.meta.env.DEV) console.log('[SkinManager] loadInfo done, skinDataUrl:', skinDataUrl.value ? 'has data' : 'null')
}

async function pickAndUpload() {
  try {
    const filePath = await selectFile('选择皮肤 PNG 文件', [{ name: 'PNG 图片', extensions: ['png'] }])
    if (!filePath) return

    uploading.value = true
    try {
      await uploadSkin(filePath, variant.value)
      showSuccess('皮肤上传成功')
      await loadInfo()
      // 触发主页头像刷新
      bumpSkinVersion()
    } catch (e) {
      showError(String(e))
    } finally {
      uploading.value = false
    }
  } catch (e) {
    showError(String(e))
  }
}

async function onEquipCape(capeId: string) {
  uploading.value = true
  try {
    await equipCape(capeId)
    showSuccess('披风已装备')
    await loadInfo()
    bumpSkinVersion()
  } catch (e) {
    showError(String(e))
  } finally {
    uploading.value = false
  }
}

async function onUnequipCape() {
  uploading.value = true
  try {
    await unequipCape()
    showSuccess('披风已取消')
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
  // 触发所有 SkinAvatar 重新加载（显示新选择的皮肤）
  bumpSkinVersion()
  showSuccess(`已切换为 ${skinName} 皮肤`)
}

/** 下载当前皮肤 PNG 到本地（弹出保存对话框） */
async function saveSkinToLocal() {
  if (!skinDataUrl.value) {
    showError('当前无皮肤数据')
    return
  }
  // 皮肤文件名：用户名_皮肤模型.png
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

function openChangePassword() {
  open('https://account.live.com/password/Change').catch(() => showError('打开网页失败'))
}

function openChangeUsername() {
  open('https://www.minecraft.net/zh-hans/msaprofile/mygames/editprofile').catch(() => showError('打开网页失败'))
}

function close() {
  emit('update:visible', false)
}

watch(() => props.visible, (v) => {
  if (v) loadInfo()
}, { immediate: true })
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
                  <SkinAvatar :uuid="uuid" :username="username" :size="40" :overlay="true" :login-type="isMicrosoft ? 'Microsoft' : 'Offline'" />
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

              <!-- 右：上传（微软）/ 皮肤选择（离线）+ 快捷入口 -->
              <div class="space-y-4">
                <!-- 微软账号：上传皮肤 -->
                <div v-if="isMicrosoft" class="rounded-lg border border-gray-100 p-4">
                  <div class="mb-3 text-sm font-medium text-gray-700">上传新皮肤</div>
                  <div class="mb-3 text-xs text-gray-500">
                    支持 64x64 或 64x32 PNG<br/>
                    文件需小于 24KB（Mojang 限制）
                  </div>
                  <div class="mb-3">
                    <label class="mb-1 block text-xs text-gray-500">皮肤模型</label>
                    <div class="flex gap-2">
                      <button
                        class="flex-1 rounded-md border px-3 py-1.5 text-xs transition-colors"
                        :class="variant === 'classic' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
                        @click="variant = 'classic'"
                      >Steve（经典）</button>
                      <button
                        class="flex-1 rounded-md border px-3 py-1.5 text-xs transition-colors"
                        :class="variant === 'slim' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
                        @click="variant = 'slim'"
                      >Alex（纤细）</button>
                    </div>
                  </div>
                  <button
                    class="w-full rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:opacity-50"
                    :disabled="uploading"
                    @click="pickAndUpload"
                  >
                    {{ uploading ? '处理中...' : '选择文件并上传' }}
                  </button>
                </div>

                <!-- 离线账号：本地皮肤选择网格 -->
                <div v-else class="rounded-lg border border-gray-100 p-4">
                  <div class="mb-3 text-sm font-medium text-gray-700">选择默认皮肤</div>
                  <div class="mb-3 text-xs text-gray-500">
                    离线账号仅支持本地显示，选择后启动器和头像将显示该皮肤。
                  </div>
                  <div class="grid grid-cols-3 gap-2">
                    <button
                      v-for="skin in defaultSkins"
                      :key="skin.name"
                      class="flex flex-col items-center rounded-md border p-2 transition-colors"
                      :class="selectedLocalSkin === skin.name
                        ? 'border-primary-500 bg-primary-50 text-primary-700'
                        : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
                      @click="onSelectLocalSkin(skin.name)"
                    >
                      <SkinAvatar
                        :skin-url="skin.url"
                        :size="48"
                        :rounded="false"
                        :overlay="true"
                      />
                      <span class="mt-1 text-[10px]">{{ skin.name }}</span>
                    </button>
                  </div>
                </div>

                <!-- 快捷入口（仅微软账号） -->
                <div v-if="isMicrosoft" class="rounded-lg border border-gray-100 p-4">
                  <div class="mb-3 text-sm font-medium text-gray-700">账号管理</div>
                  <div class="space-y-2">
                    <button
                      class="flex w-full items-center justify-between rounded-md border border-gray-200 px-3 py-2 text-xs text-gray-700 transition-colors hover:bg-gray-50"
                      @click="openChangePassword"
                    >
                      <span>修改密码</span>
                      <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
                    </button>
                    <button
                      class="flex w-full items-center justify-between rounded-md border border-gray-200 px-3 py-2 text-xs text-gray-700 transition-colors hover:bg-gray-50"
                      @click="openChangeUsername"
                    >
                      <span>修改用户名（每30天一次）</span>
                      <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
                    </button>
                  </div>
                </div>

                <!-- 动画状态（所有账号类型共用） -->
                <div class="rounded-lg border border-gray-100 p-4">
                  <div class="mb-3 text-sm font-medium text-gray-700">动画状态</div>
                  <div class="flex flex-wrap gap-1.5">
                    <Tooltip
                      v-for="opt in animationOptions"
                      :key="opt.value"
                      :text="opt.label"
                      position="top"
                      :delay="0"
                    >
                      <button
                        class="flex h-8 w-8 items-center justify-center rounded-md border transition-colors"
                        :class="animation === opt.value
                          ? 'border-primary-500 bg-primary-50 text-primary-700'
                          : 'border-gray-200 text-gray-500 hover:bg-gray-50'"
                        @click="animation = opt.value"
                      >
                        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                          <path v-for="(d, i) in opt.paths" :key="i" :d="d" />
                        </svg>
                      </button>
                    </Tooltip>
                  </div>
                </div>
              </div>

              <!-- 披风列表（仅微软账号，横跨两列） -->
              <div v-if="isMicrosoft" class="rounded-lg border border-gray-100 p-4 md:col-span-2">
                <div class="mb-3 flex items-center justify-between">
                  <div class="text-sm font-medium text-gray-700">披风列表</div>
                  <button
                    v-if="activeCape"
                    class="rounded-md border border-red-200 px-2 py-1 text-xs text-red-500 transition-colors hover:bg-red-50 disabled:opacity-50"
                    :disabled="uploading"
                    @click="onUnequipCape"
                  >取消当前披风</button>
                </div>
                <div v-if="info && info.capes.length > 0" class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
                  <button
                    v-for="cape in info.capes"
                    :key="cape.id"
                    class="flex items-center gap-2 rounded-md border px-2 py-2 text-left text-xs transition-colors disabled:opacity-50"
                    :class="cape.state === 'ACTIVE' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-700 hover:bg-gray-50'"
                    :disabled="uploading || cape.state === 'ACTIVE'"
                    @click="onEquipCape(cape.id)"
                  >
                    <svg class="h-4 w-4 flex-none" viewBox="0 0 20 20" fill="currentColor">
                      <path v-if="cape.state === 'ACTIVE'" fill-rule="evenodd" d="M16.7 5.3a1 1 0 010 1.4l-8 8a1 1 0 01-1.4 0l-4-4a1 1 0 011.4-1.4L8 12.6l7.3-7.3a1 1 0 011.4 0z" clip-rule="evenodd" />
                      <path v-else d="M3 5a2 2 0 012-2h10a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm2 1a1 1 0 011-1h8a1 1 0 110 2H6a1 1 0 01-1-1z" />
                    </svg>
                    <span class="flex-1 truncate">{{ cape.display_name }}</span>
                  </button>
                </div>
                <div v-else class="py-6 text-center text-xs text-gray-400">暂无披风</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
