<script setup lang="ts">
/**
 * 版本选择入口（参考 PCL2 PageLaunchLeft 的 BtnVersion）
 *
 * 显示当前选中的版本（方块图标 + 版本名 + 类型），点击跳转到版本选择页。
 * 不再使用下拉框，版本选择在独立的 /select 页面完成。
 */

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'
import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

const router = useRouter()
const versionStore = useVersionStore()

/** 推断版本类型（仅根据 ID 字符串匹配） */
function inferVersionType(id: string): string {
  if (!id) return 'release'
  const lower = id.toLowerCase()
  if (lower.includes('neoforge')) return 'neoforge'
  if (lower.includes('forge')) return 'forge'
  if (lower.includes('fabric')) return 'fabric'
  if (lower.includes('optifine')) return 'optifine'
  if (lower.includes('liteloader')) return 'liteloader'
  if (/^\d{2}w\d{2}[a-z]/.test(id)) return 'snapshot'
  return 'release'
}

interface TypeMeta {
  icon: string
  label: string
}
const typeMetaMap: Record<string, TypeMeta> = {
  release:    { icon: grassIcon,        label: '正式版' },
  snapshot:   { icon: commandBlockIcon, label: '快照' },
  forge:      { icon: anvilIcon,        label: 'Forge' },
  neoforge:   { icon: neoforgeIcon,     label: 'NeoForge' },
  fabric:     { icon: fabricIcon,       label: 'Fabric' },
  optifine:   { icon: optifineIcon,     label: 'OptiFine' },
  liteloader: { icon: liteloaderIcon,   label: 'LiteLoader' },
  old:        { icon: cobblestoneIcon,  label: '旧版' },
  fool:       { icon: goldBlockIcon,    label: '愚人节版' },
}
const defaultMeta: TypeMeta = { icon: grassIcon, label: '其他' }

const selectedId = computed(() => versionStore.selectedVersion)
const currentMeta = computed<TypeMeta>(() => {
  if (!selectedId.value) return defaultMeta
  return typeMetaMap[inferVersionType(selectedId.value)] ?? defaultMeta
})

function goToSelect() {
  router.push('/select')
}
</script>

<template>
  <button
    class="flex h-[35px] min-w-0 flex-1 items-center justify-between overflow-hidden rounded-[3px] border border-gray-300 bg-white/80 px-3 text-[13px] text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
    @click="goToSelect"
  >
    <div class="flex min-w-0 flex-1 items-center gap-2 overflow-hidden">
      <img
        :src="currentMeta.icon"
        class="h-4 w-4 flex-none rounded-sm"
        alt=""
      >
      <span v-if="selectedId" class="min-w-0 flex-1 truncate">{{ selectedId }}</span>
      <span v-else class="text-gray-400">无可用版本</span>
    </div>
    <div class="flex flex-none items-center gap-1.5">
      <span v-if="selectedId" class="text-xs text-gray-400">{{ currentMeta.label }}</span>
      <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M7.3 14.7a1 1 0 010-1.4L11.6 9 7.3 4.7a1 1 0 011.4-1.4l5 5a1 1 0 010 1.4l-5 5a1 1 0 01-1.4 0z" clip-rule="evenodd" />
      </svg>
    </div>
  </button>
</template>
