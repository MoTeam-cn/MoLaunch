<script setup lang="ts">
/**
 * 从存档选择 NBT 文件抽屉
 *
 * 选版本 → 选存档 → 列出存档内 NBT 文件（level.dat / playerdata / region .mca），
 * 选中后 emit select 返回文件绝对路径。复用 archiveList / nbtListSaveFiles 后端能力。
 */
import { ref, computed, watch, defineAsyncComponent } from 'vue'
import { FolderOpenIcon, InboxIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { archiveList, nbtListSaveFiles } from '@/utils/api/tools'
import type { ArchiveItem, NbtSaveFileItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType } from '@/utils/api/version'
import type { InstalledVersionInfo } from '@/utils/api/version'
import { toastError } from '@/utils/toast'

const props = withDefaults(defineProps<{ visible?: boolean }>(), { visible: false })

const emit = defineEmits<{
  'update:visible': [v: boolean]
  select: [payload: { path: string; name: string; rel_path: string; kind: string }]
}>()

const versions = ref<InstalledVersionInfo[]>([])
const versionsLoading = ref(false)
const selectedVersionId = ref('')
const saves = ref<ArchiveItem[]>([])
const savesLoading = ref(false)
const selectedWorld = ref('')
const files = ref<NbtSaveFileItem[]>([])
const filesLoading = ref(false)
const selectedPath = ref('')

const versionOptions = computed(() =>
  versions.value.map((v) => ({
    label: `${v.id}${v.version_type === 'modded' ? ' (modded)' : ''}`,
    value: v.id,
  })),
)

const worldOptions = computed(() =>
  saves.value.map((s) => ({ label: s.name, value: s.name })),
)

const kindMeta: Record<string, { label: string; color: string }> = {
  level: { label: '存档数据', color: 'blue' },
  player: { label: '玩家数据', color: 'green' },
  region: { label: '区块', color: 'purple' },
  other: { label: '其他', color: 'gray' },
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

async function loadVersions() {
  if (!props.visible) return
  versionsLoading.value = true
  try {
    versions.value = await listInstalledVersionsWithType()
  } catch (e) {
    toastError('获取版本列表失败: ' + String(e))
  } finally {
    versionsLoading.value = false
  }
}

async function loadSaves(versionId: string) {
  savesLoading.value = true
  selectedWorld.value = ''
  saves.value = []
  files.value = []
  selectedPath.value = ''
  try {
    const res = await archiveList(versionId)
    saves.value = res.items
  } catch (e) {
    toastError('获取存档列表失败: ' + String(e))
  } finally {
    savesLoading.value = false
  }
}

async function loadFiles(worldName: string) {
  filesLoading.value = true
  files.value = []
  selectedPath.value = ''
  try {
    const res = await nbtListSaveFiles(worldName, selectedVersionId.value || undefined)
    files.value = res.items
  } catch (e) {
    toastError('获取存档 NBT 文件失败: ' + String(e))
  } finally {
    filesLoading.value = false
  }
}

watch(() => props.visible, loadVersions)
watch(selectedVersionId, (vid) => {
  if (vid) loadSaves(vid)
})
watch(selectedWorld, (w) => {
  if (w) loadFiles(w)
})

function onConfirm() {
  const item = files.value.find((f) => f.path === selectedPath.value)
  if (!item) return
  emit('select', {
    path: item.path,
    name: item.name,
    rel_path: item.rel_path,
    kind: item.kind,
  })
  emit('update:visible', false)
}

function handleCancel() {
  emit('update:visible', false)
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    title="从存档选择 NBT 文件"
    :width="460"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-4 p-5">
      <div>
        <div class="mb-1.5 text-sm font-medium text-gray-700">选择版本</div>
        <Select
          v-model="selectedVersionId"
          :options="versionOptions"
          placeholder="选择游戏版本（用于定位存档目录）"
        />
      </div>

      <div>
        <div class="mb-1.5 text-sm font-medium text-gray-700">选择存档</div>
        <div class="flex items-center gap-2">
          <Select
            v-model="selectedWorld"
            :options="worldOptions"
            placeholder="选择存档"
            :disabled="saves.length === 0"
          />
          <ArrowPathIcon
            v-if="savesLoading"
            class="h-4 w-4 flex-none animate-spin text-gray-400"
          />
        </div>
      </div>

      <div>
        <div class="mb-1.5 text-sm font-medium text-gray-700">NBT 文件</div>
        <div
          v-if="filesLoading"
          class="flex h-28 flex-col items-center justify-center gap-2 text-sm text-gray-400"
        >
          <ArrowPathIcon class="h-5 w-5 animate-spin" />
          正在扫描存档文件…
        </div>
        <div
          v-else-if="files.length"
          class="max-h-72 overflow-y-auto rounded-lg border border-gray-200"
        >
          <div
            v-for="f in files"
            :key="f.path"
            class="flex cursor-pointer items-center gap-2 px-3 py-2 text-sm transition-colors hover:bg-gray-50"
            :class="selectedPath === f.path ? 'bg-primary-50' : ''"
            @click="selectedPath = f.path"
          >
            <FolderOpenIcon class="h-4 w-4 flex-none text-gray-400" />
            <div class="min-w-0 flex-1">
              <div class="truncate text-gray-800">{{ f.rel_path }}</div>
              <div class="text-xs text-gray-400">{{ formatSize(f.size) }}</div>
            </div>
            <Tag size="small" :color="kindMeta[f.kind]?.color ?? 'gray'">
              {{ kindMeta[f.kind]?.label ?? f.kind }}
            </Tag>
          </div>
        </div>
        <div v-else class="flex h-28 flex-col items-center justify-center gap-2 text-sm text-gray-400">
          <InboxIcon class="h-8 w-8 text-gray-300" />
          请先选择版本与存档
        </div>
      </div>

      <div class="flex gap-3 pt-1">
        <Button type="secondary" class="flex-1" @click="handleCancel">取消</Button>
        <Button class="flex-1" :disabled="!selectedPath" @click="onConfirm">选择该文件</Button>
      </div>
    </div>
  </Drawer>
</template>
