<script setup lang="ts">
/**
 * 实例快照面板：创建 / 差异预览 / 一键回滚 / 删除
 *
 * 快照记录 mods 目录清单，回滚时按差异补回或禁用文件；
 * 无法找回的文件（已删除且没有禁用副本）会在结果里单独提示。
 */
import { computed, defineAsyncComponent, ref } from 'vue'
import { ClockIcon, EyeIcon, PlusIcon, TrashIcon, ArrowUturnLeftIcon } from '@heroicons/vue/24/outline'
import {
  createModSnapshot,
  deleteModSnapshot,
  diffModSnapshot,
  listModSnapshots,
  restoreModSnapshot,
  type SnapshotDiff,
  type SnapshotMeta,
} from '@/utils/api/mod-snapshot'
import { showConfirmAsync } from '@/utils/modal'
import { toastError, toastSuccess, toastWarning } from '@/utils/toast'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))

const props = defineProps<{ versionId: string }>()
const emit = defineEmits<{ restored: [] }>()

const visible = ref(false)
const loading = ref(false)
const busyId = ref<string | null>(null)
const snapshots = ref<SnapshotMeta[]>([])
const diff = ref<SnapshotDiff | null>(null)

const kindLabels: Record<string, string> = {
  manual: '手动创建',
  'auto-before-install': '安装前自动',
  'auto-before-update': '更新前自动',
  'auto-before-delete': '删除前自动',
}

function kindLabel(kind: string): string {
  return kindLabels[kind] ?? kind
}

function formatTime(ms: number): string {
  const date = new Date(ms)
  if (Number.isNaN(date.getTime())) return '-'
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

const latest = computed(() => snapshots.value[0] ?? null)

async function refresh() {
  if (!props.versionId) return
  loading.value = true
  try {
    snapshots.value = await listModSnapshots(props.versionId)
  } catch (error) {
    toastError(`读取快照失败：${String(error)}`)
  } finally {
    loading.value = false
  }
}

async function open() {
  diff.value = null
  visible.value = true
  await refresh()
}

async function onCreate() {
  try {
    await createModSnapshot(props.versionId, 'manual')
    toastSuccess('已创建快照')
    await refresh()
  } catch (error) {
    toastError(`创建快照失败：${String(error)}`)
  }
}

async function onPreview(target: SnapshotMeta) {
  busyId.value = target.id
  try {
    diff.value = await diffModSnapshot(props.versionId, target.id)
  } catch (error) {
    toastError(`读取差异失败：${String(error)}`)
  } finally {
    busyId.value = null
  }
}

async function onRestore(target: SnapshotMeta) {
  const ok = await showConfirmAsync(
    '回滚到这个快照？',
    `将把 mods 目录恢复到 ${formatTime(target.created_at)} 的状态：缺失的文件会重新启用，多出来的文件会被禁用（不删除，可再启用）。`,
  )
  if (!ok) return
  busyId.value = target.id
  try {
    const result = await restoreModSnapshot(props.versionId, target.id)
    if (result.unresolved.length > 0) {
      toastWarning(`已回滚，但 ${result.unresolved.length} 个文件无法找回：${result.unresolved.slice(0, 3).join('、')}`)
    } else {
      toastSuccess(`已回滚：恢复 ${result.restored.length} 个，禁用 ${result.disabled.length} 个`)
    }
    emit('restored')
    await refresh()
  } catch (error) {
    toastError(`回滚失败：${String(error)}`)
  } finally {
    busyId.value = null
  }
}

async function onDelete(target: SnapshotMeta) {
  const ok = await showConfirmAsync('删除这个快照？', '删除后无法用它回滚，当前 mods 目录不受影响。')
  if (!ok) return
  busyId.value = target.id
  try {
    await deleteModSnapshot(props.versionId, target.id)
    toastSuccess('已删除快照')
    if (diff.value?.id === target.id) diff.value = null
    await refresh()
  } catch (error) {
    toastError(`删除失败：${String(error)}`)
  } finally {
    busyId.value = null
  }
}
</script>

<template>
  <div class="flex items-center justify-between gap-3 px-1 pb-3 text-xs text-gray-500">
    <span>
      快照记录 mods 目录清单，装完 Mod 出问题可以一键回到之前的状态
      <template v-if="latest">
        ；最近一次：{{ kindLabel(latest.kind) }} {{ formatTime(latest.created_at) }}
      </template>
    </span>
    <Button size="small" variant="secondary" :disabled="!versionId" @click="open">
      <ClockIcon class="mr-1 h-3.5 w-3.5" />
      快照与回滚
    </Button>
  </div>

  <Drawer :visible="visible" placement="right" :width="520" :z-index="1250" @update:visible="visible = $event">
    <template #title>
      <span class="text-sm font-semibold text-gray-900">快照与回滚</span>
    </template>

    <div class="space-y-3 p-1">
      <div class="flex items-center justify-between gap-3">
        <p class="text-xs text-gray-500">最多保留最近 10 个快照</p>
        <Button size="small" @click="onCreate">
          <PlusIcon class="mr-1 h-3.5 w-3.5" />
          立即创建
        </Button>
      </div>

      <p v-if="loading" class="py-6 text-center text-xs text-gray-400">加载中…</p>
      <p v-else-if="snapshots.length === 0" class="py-6 text-center text-xs text-gray-400">
        还没有快照，点「立即创建」记录当前 mods 目录
      </p>

      <ul v-else class="space-y-2">
        <li
          v-for="item in snapshots"
          :key="item.id"
          class="rounded-lg border border-gray-200 p-3"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="min-w-0">
              <p class="truncate text-sm text-gray-900">
                {{ kindLabel(item.kind) }} · {{ formatTime(item.created_at) }}
              </p>
              <p class="mt-0.5 text-xs text-gray-500">{{ item.files.length }} 个 Mod 文件</p>
            </div>
            <div class="flex flex-none items-center gap-1">
              <Button size="small" variant="text" :disabled="busyId === item.id" @click="onPreview(item)">
                <EyeIcon class="h-3.5 w-3.5" />
              </Button>
              <Button size="small" variant="text" :disabled="busyId === item.id" @click="onRestore(item)">
                <ArrowUturnLeftIcon class="h-3.5 w-3.5" />
              </Button>
              <Button size="small" variant="text" :disabled="busyId === item.id" @click="onDelete(item)">
                <TrashIcon class="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>

          <div v-if="diff && diff.id === item.id" class="mt-2 space-y-1 rounded-md bg-gray-50 p-2 text-xs">
            <p v-if="diff.missing.length === 0 && diff.added.length === 0 && diff.toggled.length === 0" class="text-gray-500">
              与当前状态一致，无需回滚
            </p>
            <p v-if="diff.missing.length" class="text-amber-600">
              缺少 {{ diff.missing.length }} 个：{{ diff.missing.slice(0, 5).join('、') }}
            </p>
            <p v-if="diff.added.length" class="text-rose-600">
              多出 {{ diff.added.length }} 个：{{ diff.added.slice(0, 5).join('、') }}
            </p>
            <p v-if="diff.toggled.length" class="text-gray-600">
              启用状态不同 {{ diff.toggled.length }} 个：{{ diff.toggled.slice(0, 5).join('、') }}
            </p>
          </div>
        </li>
      </ul>
    </div>
  </Drawer>
</template>
