/**
 * 前置 Mod 依赖检查与安装 composable
 *
 * 封装 check_mod_dependencies 和 install_mod_with_dependencies 两个 IPC 调用，
 * 提供状态管理（checking / installing / missing / upToDate）。
 *
 * 调用方：
 * - ResourceDetail.vue：下载前检查，弹窗确认后一键安装主 mod + 勾选前置
 * - useModUpdate.ts：更新前检查（阶段 4 集成）
 */
import { ref } from 'vue'
import { versionModsManager, VERSION_MODS_ACTIONS } from '@/utils/api/version-mods-manager'
import type {
  ResourceVersion,
  ResolvedDependency,
  DependencyCheckResult,
  DependencyInstallResult,
} from '@/types/community'

/** check_mod_dependencies 入参 */
export interface CheckDepsParams {
  versionId: string
  platform: string
  modVersion: ResourceVersion
  gameVersion: string
  modLoader: number
}

/** install_mod_with_dependencies 入参 */
export interface InstallDepsParams {
  versionId: string
  mainVersion: ResourceVersion
  deps: ResolvedDependency[]
}

export function useDependencyCheck() {
  const checking = ref(false)
  const installing = ref(false)
  const missing = ref<ResolvedDependency[]>([])
  const upToDate = ref<ResolvedDependency[]>([])

  /**
   * 检查前置依赖
   *
   * @returns true=有缺失前置需安装，false=无缺失可直接下载
   */
  async function check(params: CheckDepsParams): Promise<boolean> {
    checking.value = true
    try {
      const result = await versionModsManager<DependencyCheckResult>(
        VERSION_MODS_ACTIONS.CHECK_MOD_DEPENDENCIES,
        params,
      )
      missing.value = result.missing
      upToDate.value = result.upToDate
      return result.missing.length > 0
    } finally {
      checking.value = false
    }
  }

  /**
   * 批量安装主 mod + 用户勾选的前置
   */
  async function install(params: InstallDepsParams): Promise<DependencyInstallResult> {
    installing.value = true
    try {
      return await versionModsManager<DependencyInstallResult>(
        VERSION_MODS_ACTIONS.INSTALL_MOD_WITH_DEPENDENCIES,
        params,
      )
    } finally {
      installing.value = false
    }
  }

  /** 重置状态（关闭弹窗时调用） */
  function reset() {
    missing.value = []
    upToDate.value = []
  }

  return { checking, installing, missing, upToDate, check, install, reset }
}
