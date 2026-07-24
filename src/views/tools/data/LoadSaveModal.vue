<script setup lang="ts">
/**
 * 从存档加载种子弹窗
 *
 * 用于种子地图工具"从存档加载"功能：
 * 1. 选择已安装版本 → 自动拉取该版本 saves 列表
 * 2. 选择存档 → 确认 → 后端读 level.dat 提取种子
 * 3. 同时解析版本 JSON 拿 MC 版本号 → 自动映射到 cubiomes 枚举
 * 4. emit('load', { seed, mcVersion }) 给父组件加载地图
 *
 * 复用：archiveList / extractSaveSeed（tools API）、listInstalledVersionsWithType
 * （version API）、getVersionGameVersion（personalization API）、mapMcVersionToCubiomes
 * （useSeedMap）、Button/Select/Tooltip 自定义组件。
 */
import { ref, watch } from 'vue'
import { FolderOpenIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { archiveList, extractSaveSeed, type ArchiveItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { getVersionGameVersion } from '@/utils/api/personalization'
import { mapMcVersionToCubiomes } from './useSeedMap'
import { toastError } from '@/utils/toast'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'load', payload: { seed: string; mcVersion: number; worldName: string }): void
}>()

// ===== 版本列表 =====
const versions = ref<InstalledVersionInfo[]>([])
const versionsLoading = ref(false)
const selectedVersionId = ref<string>('')

// ===== 存档列表 =====
const saves = ref<ArchiveItem[]>([])
const savesLoading = ref(false)
const selectedWorld = ref<string>('')

// ===== 确认加载 =====
const loading = ref(false)

// 弹窗打开时拉取版本列表
watch(
  () => props.visible,
  async (v) => {
    if (!v) return
    selectedVersionId.value = ''
    selectedWorld.value = ''
    saves.value = []
    versionsLoading.value = true
    try {
      versions.value = await listInstalledVersionsWithType()
    } catch (e) {
      toastError('获取版本列表失败: ' + (e instanceof Error ? e.message : String(e)))
    } finally {
      versionsLoading.value = false
    }
  },
)

// 选版本后拉取 saves 列表
watch(selectedVersionId, async (vid) => {
  selectedWorld.value = ''
  saves.value = []
  if (!vid) return
  savesLoading.value = true
  try {
    const res = await archiveList(vid)
    saves.value = res.items.filter((s) => s.has_level_dat)
  } catch (e) {
    toastError('获取存档列表失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    savesLoading.value = false
  }
})

const versionOptions = () =>
  versions.value.map((v) => ({ label: v.id, value: v.id }))

const saveOptions = () =>
  saves.value.map((s) => ({ label: s.name, value: s.name }))

async function onConfirm() {
  if (!selectedVersionId.value || !selectedWorld.value) return
  loading.value = true
  try {
    // 1. 提取种子
    const seedResult = await extractSaveSeed(selectedWorld.value, selectedVersionId.value)
    // 2. 解析版本号并映射 cubiomes 枚举
    let mcVersion = 28 // 默认最新（MC_26_2 = MC_NEWEST = 28）
    try {
      const gameVer = await getVersionGameVersion(selectedVersionId.value)
      if (gameVer) {
        const mapped = mapMcVersionToCubiomes(gameVer)
        if (mapped !== null) mcVersion = mapped
      }
    } catch {
      // 版本号解析失败时用默认值，不阻塞加载
    }
    emit('load', {
      seed: seedResult.seed,
      mcVersion,
      worldName: selectedWorld.value,
    })
  } catch (e) {
    toastError('提取种子失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="handleCancel"
      >
        <div class="mx-4 w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
          <!-- 标题 -->
          <div class="mb-4 flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary-50">
              <FolderOpenIcon class="h-5 w-5 text-primary-500" />
            </div>
            <h3 class="text-lg font-semibold text-gray-900">从存档加载种子</h3>
          </div>

          <!-- 版本选择 -->
          <div class="mb-4">
            <label class="mb-1.5 block text-xs font-medium text-gray-700">选择版本</label>
            <Select
              v-model="selectedVersionId"
              :options="versionOptions()"
              placeholder="选择已安装的版本"
              class="w-full"
            />
            <div v-if="versionsLoading" class="mt-1.5 flex items-center gap-1 text-xs text-gray-400">
              <ArrowPathIcon class="h-3 w-3 animate-spin" />
              加载版本列表...
            </div>
          </div>

          <!-- 存档选择 -->
          <div class="mb-5">
            <label class="mb-1.5 block text-xs font-medium text-gray-700">选择存档</label>
            <Select
              v-model="selectedWorld"
              :options="saveOptions()"
              :placeholder="selectedVersionId ? '选择该版本的存档' : '请先选择版本'"
              :disabled="!selectedVersionId || savesLoading"
              class="w-full"
            />
            <div v-if="savesLoading" class="mt-1.5 flex items-center gap-1 text-xs text-gray-400">
              <ArrowPathIcon class="h-3 w-3 animate-spin" />
              加载存档列表...
            </div>
            <p v-else-if="selectedVersionId && !savesLoading && saves.length === 0" class="mt-1.5 text-xs text-gray-400">
              该版本目录下暂无有效存档（需含 level.dat）
            </p>
          </div>

          <!-- 操作按钮 -->
          <div class="flex justify-end gap-2">
            <Button type="ghost" size="small" @click="handleCancel">取消</Button>
            <Button
              type="primary"
              size="small"
              :loading="loading"
              :disabled="!selectedVersionId || !selectedWorld"
              @click="onConfirm"
            >
              加载种子
            </Button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
