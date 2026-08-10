/**
 * 资源包/光影详情查询 composable：详情按钮三级 fallback
 *
 * 1. project 已预加载 → 直接弹窗
 * 2. 预加载未完成 → 等待 preload done（最多 3s），期间 project 就绪则弹窗
 * 3. 无 project → 本地信息弹窗
 *
 * packs 为 zip 包无 JAR 元数据（无 slug），在线详情只能依赖 hash 匹配到的 project。
 */
import { ref, type Ref } from 'vue'
import { showInfo } from '@/utils/modal'
import { formatBytes } from '@/utils/format'
import type { ResourceProject } from '@/types/community'
import type { PackInfo } from '@/utils/api/personalization'

export function usePackDetailQuery() {
  // 详情弹窗（关联到 CF/MR 平台工程时使用）
  const detailVisible = ref(false)
  const detailProject = ref<ResourceProject | null>(null)
  /** 当前正在加载详情的包 file_name（用于按钮 spinner + 防止重复点击同一包） */
  const detailLoadingFor = ref<string | null>(null)

  /** 显示本地信息弹窗（无法关联到 CF/MR 平台时使用） */
  function showLocalPackInfo(pack: PackInfo) {
    const lines: string[] = []
    lines.push(`文件：${pack.file_name}（${formatBytes(pack.size)}）`)
    lines.push(`状态：${pack.is_enabled ? '已启用' : '已禁用'}${pack.is_folder ? '（文件夹）' : ''}`)
    lines.push('')
    lines.push('该包未匹配到 CurseForge / Modrinth 平台工程，可在资源管理器中查看。')
    showInfo(pack.enabled_name || pack.file_name, lines.join('\n'))
  }

  /**
   * 详情按钮：不主动发网络请求，只判断 `pack.project` 是否已被预加载填充
   *
   * 三级 fallback：
   * 1. project 已就绪 → 直接弹 ResourceDetail
   * 2. 预加载未完成 → 等待 `packs-preload-done`（最多 3s），期间 project 就绪则弹窗
   * 3. preload done 仍无 project → 本地信息弹窗
   */
  async function handleShowInfo(
    pack: PackInfo,
    packs: Ref<PackInfo[]>,
    isPreloadDone: Ref<boolean>,
  ) {
    // 防呆：同一包正在加载中，忽略重复点击
    if (detailLoadingFor.value === pack.file_name) return

    // 1. 零延迟路径：预加载已就绪，直接弹窗
    if (pack.project) {
      detailProject.value = pack.project
      detailVisible.value = true
      return
    }

    // 2. 等待预加载完成（project 可能在等待期间被填充）
    detailLoadingFor.value = pack.file_name
    try {
      for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 100))
        const current = packs.value.find(p => p.file_name === pack.file_name)
        if (current?.project) {
          detailProject.value = current.project
          detailVisible.value = true
          return
        }
        if (isPreloadDone.value) break
      }
    } finally {
      detailLoadingFor.value = null
    }

    // 3. 预加载完成仍无 project，走本地信息弹窗
    showLocalPackInfo(pack)
  }

  return { detailVisible, detailProject, detailLoadingFor, handleShowInfo }
}
