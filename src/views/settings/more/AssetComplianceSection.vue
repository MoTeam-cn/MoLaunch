<script setup lang="ts">
/**
 * 素材来源与版权投诉区块
 *
 * 特性：
 * - 说明内置第三方素材（游戏纹理、皮肤、结构图标、字体）的来源与权利归属
 * - 提供权利人联系邮箱（可一键复制）与结构化投诉模板入口
 * - 公示响应时限承诺，配合根目录 NOTICE.md 使用
 *
 * 用法：在 CreditsTab 的「法律信息」折叠卡内直接渲染 <AssetComplianceSection />。
 */
import { defineAsyncComponent } from 'vue'
import { ShieldExclamationIcon } from '@heroicons/vue/24/outline'
import { copyToClipboard } from '@/utils/clipboard'
import { openLink } from '@/utils/aboutLogos'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

/** 版权投诉专用邮箱（须与根目录 NOTICE.md 保持一致） */
const CONTACT_EMAIL = 'support@moteam.top'
/** 结构化版权投诉模板（GitHub Issue Form） */
const COMPLAINT_URL =
  'https://github.com/MoTeam-cn/MoLaunch/issues/new?template=copyright.yml'

async function onCopyEmail(): Promise<void> {
  await copyToClipboard(CONTACT_EMAIL, { toast: true })
}
</script>

<template>
  <div>
    <div class="flex items-center gap-1.5 text-[13px] font-semibold text-gray-700">
      <ShieldExclamationIcon class="h-4 w-4 text-gray-400" />
      素材来源与版权投诉
    </div>
    <p class="mt-1 text-[12px] leading-relaxed text-gray-500">
      为在界面中还原游戏内的视觉呈现，本项目使用了部分 Minecraft 游戏纹理与皮肤，其版权归 Mojang Studios 所有；
      Minecraft 是 Mojang Synergies AB 的商标。另有少量图标与字体来自第三方开源项目或社区站点，版权归各自权利人所有。
      本项目永久免费、非商业，不主张对上述素材的任何权利，完整来源清单见项目根目录 NOTICE.md。
    </p>

    <div class="mt-2 rounded-lg bg-gray-50 px-3 py-2">
      <div class="text-[11px] font-semibold text-gray-600">权利人与投诉渠道</div>
      <p class="mt-1 text-[12px] leading-relaxed text-gray-500">
        如您是上述素材的权利人，认为本项目的使用方式不当，请通过以下任一渠道联系。
        我们承诺：收到有效通知后 48 小时内确认；经核实确有不当时，7 天内在下一个版本中移除或替换；
        情况紧急时可直接下线相关资源，无需等待版本发布。
      </p>
      <div class="mt-2 flex flex-wrap items-center gap-2">
        <code
          class="rounded-sm bg-white px-1.5 py-0.5 text-[11px] text-gray-700 ring-1 ring-gray-200"
        >{{ CONTACT_EMAIL }}</code>
        <Button type="text" size="small" @click="onCopyEmail">复制邮箱</Button>
        <Button type="text" size="small" @click="openLink(COMPLAINT_URL)">提交版权投诉</Button>
      </div>
    </div>
  </div>
</template>
