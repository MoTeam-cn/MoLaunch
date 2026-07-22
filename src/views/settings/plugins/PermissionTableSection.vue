<script setup lang="ts">
/**
 * 可用权限说明表格（可展开/收起，分三组：始终允许 / 普通 / 高级）
 */
import {
  PERMISSION_REGISTRY,
  NORMAL_PERMISSIONS,
  ADVANCED_PERMISSIONS,
  RISK_STYLES,
} from '@/plugins/permissions'
import CollapsibleCard from '@/components/common/CollapsibleCard.vue'
import { ShieldCheckIcon } from '@heroicons/vue/24/outline'
</script>

<template>
  <CollapsibleCard>
    <template #title>
      <div class="flex items-center gap-2">
        <ShieldCheckIcon class="h-4 w-4 text-primary-500" />
        <span class="text-sm font-semibold text-gray-900">可用权限说明</span>
        <span class="text-xs text-gray-400">({{ PERMISSION_REGISTRY.length }} 项)</span>
      </div>
    </template>

    <!-- 始终允许 -->
    <div>
      <p class="mb-2 text-xs font-medium text-gray-700">始终允许（无需声明）</p>
      <div class="space-y-2">
        <div
          v-for="perm in PERMISSION_REGISTRY.filter((p) => p.alwaysAllowed)"
          :key="perm.name"
          class="grid grid-cols-[180px_1fr] items-start gap-3 rounded bg-gray-50 px-3 py-2"
        >
          <span class="inline-flex items-center rounded bg-gray-200 px-1.5 py-0.5 text-[10px] font-mono font-medium text-gray-600">
            {{ perm.name }}
          </span>
          <div class="min-w-0">
            <p class="text-xs text-gray-700">{{ perm.description }}</p>
            <p class="mt-0.5 text-[11px] text-gray-400">{{ perm.useCase }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 普通权限 -->
    <div class="border-t border-gray-200 mt-3 pt-3">
      <p class="mb-2 text-xs font-medium text-gray-700">普通权限（低风险，需声明）</p>
      <div class="space-y-2">
        <div
          v-for="perm in NORMAL_PERMISSIONS"
          :key="perm.name"
          class="grid grid-cols-[180px_1fr] items-start gap-3 rounded bg-blue-50/50 px-3 py-2"
        >
          <span class="inline-flex items-center rounded bg-blue-50 px-1.5 py-0.5 text-[10px] font-mono font-medium text-blue-700">
            {{ perm.name }}
          </span>
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-xs text-gray-700">{{ perm.description }}</p>
              <span
                class="inline-flex items-center rounded px-1 py-0.5 text-[9px] font-medium"
                :class="RISK_STYLES[perm.risk].bg + ' ' + RISK_STYLES[perm.risk].text"
              >
                {{ RISK_STYLES[perm.risk].label }}
              </span>
            </div>
            <p class="mt-0.5 text-[11px] text-gray-400">{{ perm.useCase }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 高级权限 -->
    <div class="border-t border-gray-200 mt-3 pt-3">
      <p class="mb-2 text-xs font-medium text-gray-700">高级权限（高风险，需声明 + 额外配置）</p>
      <div class="space-y-2">
        <div
          v-for="perm in ADVANCED_PERMISSIONS"
          :key="perm.name"
          class="grid grid-cols-[180px_1fr] items-start gap-3 rounded bg-red-50/50 px-3 py-2"
        >
          <span class="inline-flex items-center rounded bg-red-50 px-1.5 py-0.5 text-[10px] font-mono font-medium text-red-700">
            {{ perm.name }}
          </span>
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-xs text-gray-700">{{ perm.description }}</p>
              <span
                class="inline-flex items-center rounded px-1 py-0.5 text-[9px] font-medium"
                :class="RISK_STYLES[perm.risk].bg + ' ' + RISK_STYLES[perm.risk].text"
              >
                {{ RISK_STYLES[perm.risk].label }}
              </span>
            </div>
            <p class="mt-0.5 text-[11px] text-gray-400">{{ perm.useCase }}</p>
            <p v-if="perm.requiresExtraConfig" class="mt-1 text-[10px] text-red-500">
              需额外配置字段：{{ perm.requiresExtraConfig }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </CollapsibleCard>
</template>
