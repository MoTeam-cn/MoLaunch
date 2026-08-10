/**
 * 版本概览操作 composable
 *
 * 封装打开文件夹/描述/重命名/收藏/分类/图标/导出脚本/补全/删除等操作；
 * 所需 ref/computed/store 由参数注入，handler 内 toast/modal 行为与原实现保持一致。
 */
import { ref, nextTick, type ComputedRef, type Ref } from 'vue'
import type { Router } from 'vue-router'
import * as tauri from '@/utils/tauri'
import { pickSavePath } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastWarning, toastInfo } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'
import type { VersionPersonalization } from '@/utils/tauri'

interface UseVersionOverviewActionsOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 版本个性化信息（来自 useVersionSettings，可写 ref 用于原地更新字段） */
  personalization: Ref<VersionPersonalization | null>
  /** 重新加载个性化数据（来自 useVersionSettings） */
  loadPersonalization: () => Promise<void>
  /** 刷新 effectiveDir（来自 useVersionSettings） */
  refreshEffectiveDir: () => Promise<void>
  /** 路由实例（用于删除版本后跳转） */
  router: Router
  /** auth store（用于导出脚本时取用户信息） */
  authStore: {
    isLoggedIn: boolean
    currentUser: { name: string; uuid: string; access_token: string; login_type: string } | null
  }
  /** java store（用于导出脚本时取 Java 路径） */
  javaStore: {
    javaPath: string
  }
}

/**
 * 版本概览操作 composable
 *
 * 使用方式：
 * ```ts
 * const router = useRouter()
 * const authStore = useAuthStore()
 * const javaStore = useJavaStore()
 * const { selectedId, personalization, loadPersonalization, refreshEffectiveDir } = useVersionSettings()
 * const { fixing, handleRename, handleDelete, ... } = useVersionOverviewActions({
 *   selectedId, personalization, loadPersonalization, refreshEffectiveDir,
 *   router, authStore, javaStore,
 * })
 * ```
 */
export function useVersionOverviewActions(options: UseVersionOverviewActionsOptions) {
  const { selectedId, personalization, loadPersonalization, refreshEffectiveDir, router, authStore, javaStore } = options

  /** 文件补全进行中（按钮禁用 + spinner） */
  const fixing = ref(false)

  async function openFolder(path: string) {
    try {
      await tauri.openPath(path)
    } catch (e) {
      toastError('打开失败：' + String(e))
    }
  }

  function handleEditDesc() {
    if (!selectedId.value) return
    const oldDesc = personalization.value?.customInfo ?? ''
    showPrompt(
      '修改版本描述',
      '修改版本的描述文本，留空则使用默认描述。',
      async (newDesc: string) => {
        if (!selectedId.value) return
        try {
          await tauri.updateVersionPersonalization(selectedId.value, { customInfo: newDesc })
          if (personalization.value) personalization.value.customInfo = newDesc
          toastSuccess('描述已更新')
        } catch (e) {
          toastError('更新失败：' + String(e))
        }
      },
      { defaultValue: oldDesc, placeholder: '请输入版本描述' },
    )
  }

  function handleRename() {
    if (!selectedId.value) return
    showPrompt(
      '重命名版本',
      '修改版本文件夹名称（不影响游戏内版本号）',
      async (newName: string) => {
        if (!selectedId.value || !newName.trim()) return
        if (newName === selectedId.value) return
        try {
          const oldName = selectedId.value
          await tauri.renameVersion(oldName, newName.trim())
          // 等待 selectedId computed 更新
          await nextTick()
          await loadPersonalization()
          await refreshEffectiveDir()
          toastSuccess('重命名成功')
        } catch (e) {
          toastError('重命名失败：' + String(e))
        }
      },
      { defaultValue: selectedId.value, placeholder: '请输入新版本名' },
    )
  }

  async function handleToggleStar() {
    if (!selectedId.value || !personalization.value) return
    const newVal = !personalization.value.isStar
    try {
      await tauri.updateVersionPersonalization(selectedId.value, { isStar: newVal })
      personalization.value.isStar = newVal
      toastSuccess(newVal ? '已加入收藏' : '已取消收藏')
    } catch (e) {
      toastError('操作失败：' + String(e))
    }
  }

  async function handleChangeDisplayType(newType: number) {
    if (!selectedId.value || !personalization.value) return
    try {
      await tauri.updateVersionPersonalization(selectedId.value, { displayType: newType })
      personalization.value.displayType = newType
      toastSuccess('分类已更新')
    } catch (e) { toastError('更新失败：' + String(e)) }
  }

  async function handleChangeLogo(newLogo: string) {
    if (!selectedId.value || !personalization.value) return
    try {
      await tauri.updateVersionPersonalization(selectedId.value, { logo: newLogo })
      // 替换整个 personalization 对象，确保所有依赖该 ref 的组件（如首页 VersionSelector）都能响应式更新
      personalization.value = { ...personalization.value, logo: newLogo }
      toastSuccess('图标已更新')
    } catch (e) { toastError('更新失败：' + String(e)) }
  }

  async function handleExportScript() {
    if (!selectedId.value) return
    if (!authStore.isLoggedIn) return toastWarning('请先登录账号')
    const user = authStore.currentUser!
    try {
      // 按当前系统选择脚本格式（与后端 build_script_content 按 cfg(target_os) 判断保持一致）
      let isWindows = false
      try {
        isWindows = (await tauri.getSystemInfo()).os === 'windows'
      } catch {
        isWindows = navigator.platform.startsWith('Win')
      }
      const defaultName = isWindows ? `Run_${selectedId.value}.bat` : `Run_${selectedId.value}.sh`
      const filters = isWindows
        ? [{ name: '批处理文件', extensions: ['bat'] }]
        : [{ name: 'Shell 脚本', extensions: ['sh'] }]
      const savePath = await pickSavePath({ title: '选择脚本保存位置', defaultPath: defaultName, filters })
      if (!savePath) { toastInfo('已取消导出'); return }
      await tauri.exportLaunchScript(selectedId.value, user.name, user.uuid, user.login_type, javaStore.javaPath || null, savePath)
      toastSuccess('启动脚本已导出')
      // 导出后自动打开所在文件夹并选中导出的文件
      await tauri.revealInExplorer(savePath)
    } catch (e) { toastError('导出失败：' + String(e)) }
  }

  async function handleFixFiles() {
    if (!selectedId.value || fixing.value) return
    showConfirm('补全文件', `将检查并下载版本"${selectedId.value}"缺失的 libraries 和 assets 文件，可能耗时较长。`, async () => {
      fixing.value = true
      toastInfo('开始补全文件...')
      try {
        await tauri.fixVersionFiles(selectedId.value!)
        toastSuccess('文件补全完成')
      } catch (e) { toastError('补全失败：' + String(e)) }
      finally { fixing.value = false }
    })
  }

  /** 加载器检测/重装进度抽屉开关 */
  const repairDrawerOpen = ref(false)

  /** 打开加载器检测抽屉（扫描、询问、重装流程由抽屉内部驱动） */
  function handleRepairLoader() {
    if (!selectedId.value) return
    repairDrawerOpen.value = true
  }

  function handleDelete() {
    if (!selectedId.value) return
    showConfirm('删除版本', `确定要删除版本"${selectedId.value}"吗？此操作不可恢复。`, async () => {
      try {
        await tauri.uninstallVersion(selectedId.value!)
        toastSuccess('版本已删除')
        router.push('/apps')
      } catch (e) { toastError(String(e)) }
    })
  }

  return {
    fixing,
    repairDrawerOpen,
    openFolder,
    handleEditDesc,
    handleRename,
    handleToggleStar,
    handleChangeDisplayType,
    handleChangeLogo,
    handleExportScript,
    handleFixFiles,
    handleRepairLoader,
    handleDelete,
  }
}
