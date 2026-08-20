/**
 * 版本导出 Tab composable
 *
 * 封装导出选项加载、ini 配置读写、勾选联动、整合包导出与 export-progress 进度监听；
 * 选项列表原地修改 checked 字段，避免重新渲染整个树。
 */
import { ref, computed, onMounted, type ComputedRef } from 'vue'
import { useTauriEvent } from '@/utils/tauriEvent'
import {
  getExportOptions,
  exportModpack,
  saveExportConfig,
  loadExportConfig,
  EXPORT_FORMAT_OPTIONS,
  findExportFormat,
  EXPORT_PROGRESS_EVENT,
  type ExportOption,
  type ExportFormat,
  type ExportProgress,
  type ExportStage,
} from '@/utils/api/version-export-manager'
import { pickSavePath, pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastWarning, toastInfo } from '@/utils/toast'

interface UseExportTabOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
}

/**
 * 版本导出 Tab composable
 *
 * 使用方式：
 * ```ts
 * const { selectedId } = useVersionSettings()
 * const { loading, options, packName, handleExport, ... } = useExportTab({ selectedId })
 * ```
 */
export function useExportTab(options: UseExportTabOptions) {
  const { selectedId } = options

  /** 选项加载中 */
  const loading = ref(false)
  /** 导出进行中 */
  const exporting = ref(false)
  /** 所有导出选项（含子选项，原样保留层级关系） */
  const exportOptions = ref<ExportOption[]>([])
  /** 整合包名称 */
  const packName = ref('')
  /** 整合包版本号 */
  const packVersion = ref('1.0.0')
  /** 导出格式（默认 Modrinth） */
  const exportFormat = ref<ExportFormat>('modrinth')
  /** 是否联网检查 mod 下载地址 */
  const checkHostedAssets = ref(true)
  /** 仅从 Modrinth 查询（跳过 CurseForge） */
  const modrinthUploadMode = ref(false)

  /** 导出进度百分比（0-100，导出未开始时为 0） */
  const exportProgress = ref(0)
  /** 导出进度阶段（用于按阶段展示不同文案/颜色） */
  const exportStage = ref<ExportStage | null>(null)
  /** 导出进度描述文案 */
  const exportMessage = ref('')

  /** 当前格式的元信息 */
  const currentFormatMeta = computed(() => findExportFormat(exportFormat.value))
  /** 当前格式是否支持联网检查 */
  const supportsOnlineCheck = computed(() => currentFormatMeta.value.supportsOnlineCheck)
  /** 所有可选的导出格式 */
  const formatOptions = EXPORT_FORMAT_OPTIONS

  /** 可见选项（visible=false 的不展示） */
  const visibleOptions = computed(() => exportOptions.value.filter(o => o.visible))

  /** 顶层选项（无 parent） */
  const topLevelOptions = computed(() => visibleOptions.value.filter(o => !o.parent))

  /** 获取指定父选项的可见子选项列表 */
  function getChildren(parentId: string): ExportOption[] {
    return visibleOptions.value.filter(o => o.parent === parentId)
  }

  const { start } = useTauriEvent<ExportProgress>(EXPORT_PROGRESS_EVENT, (payload) => {
    // 只接受当前选中版本的事件（避免切换版本时残留旧事件）
    if (payload.versionId !== selectedId.value) return
    exportStage.value = payload.stage
    exportProgress.value = payload.percent
    exportMessage.value = payload.message
  })

  /** 重置进度状态（导出开始前/结束后调用） */
  function resetProgress() {
    exportProgress.value = 0
    exportStage.value = null
    exportMessage.value = ''
  }

  /** 加载导出选项（进入 Tab 时调用） */
  async function loadOptions() {
    if (!selectedId.value) return
    loading.value = true
    try {
      exportOptions.value = await getExportOptions(selectedId.value)
      // 默认 packName 用版本 ID（仅当用户未输入时）
      if (!packName.value) packName.value = selectedId.value
    } catch (e) {
      toastError('加载导出选项失败：' + String(e))
    } finally {
      loading.value = false
    }
  }

  /** 切换选项勾选状态（必选项禁用切换） */
  function toggleOption(opt: ExportOption) {
    if (!opt.enabled) return
    opt.checked = !opt.checked
  }

  /**
   * 将 LoadConfigResult.rulesOverride（"id=true|id=false"）应用到选项列表
   *
   * 复刻后端 config.rs::apply_config_to_options 的逻辑：
   * - 必选项（enabled=false）不允许取消，强制保持 checked=true
   * - 仅更新当前 options 列表中存在的 id
   */
  function applyConfigOverride(rulesOverride: string[]) {
    for (const rule of rulesOverride) {
      const eq = rule.indexOf('=')
      if (eq < 0) continue
      const id = rule.substring(0, eq)
      const valueStr = rule.substring(eq + 1)
      const checked = valueStr === 'true' || valueStr === '1'
      const target = exportOptions.value.find(o => o.id === id)
      if (!target) continue
      // 必选项不允许取消
      if (!target.enabled && !checked) {
        target.checked = true
      } else {
        target.checked = checked
      }
    }
  }

  /** 保存当前配置到 .ini 文件 */
  async function handleSaveConfig() {
    const savePath = await pickSavePath({
      title: '保存导出配置',
      defaultPath: `${packName.value || 'export'}.ini`,
      filters: [{ name: 'INI 配置文件', extensions: ['ini'] }],
    })
    if (!savePath) { toastInfo('已取消保存'); return }
    try {
      await saveExportConfig({
        configPath: savePath,
        packName: packName.value,
        packVersion: packVersion.value,
        checkHostedAssets: checkHostedAssets.value,
        modrinthUploadMode: modrinthUploadMode.value,
        packPath: null,
        options: exportOptions.value,
      })
      toastSuccess('配置已保存')
    } catch (e) {
      toastError('保存配置失败：' + String(e))
    }
  }

  /** 从 .ini 文件读取配置并应用到当前选项 */
  async function handleLoadConfig() {
    const file = await pickFile({
      title: '读取导出配置',
      filters: [{ name: 'INI 配置文件', extensions: ['ini'] }],
    })
    if (!file) { toastInfo('已取消读取'); return }
    try {
      const cfg = await loadExportConfig(file)
      if (cfg.packName) packName.value = cfg.packName
      if (cfg.packVersion) packVersion.value = cfg.packVersion
      checkHostedAssets.value = cfg.checkHostedAssets
      modrinthUploadMode.value = cfg.modrinthUploadMode
      applyConfigOverride(cfg.rulesOverride)
      toastSuccess('配置已读取')
    } catch (e) {
      toastError('读取配置失败：' + String(e))
    }
  }

  /** 执行整合包导出 */
  async function handleExport() {
    if (!selectedId.value) return
    if (!packName.value.trim()) {
      toastWarning('请输入整合包名称')
      return
    }
    if (!packVersion.value.trim()) {
      toastWarning('请输入整合包版本号')
      return
    }

    const meta = currentFormatMeta.value
    const savePath = await pickSavePath({
      title: '选择导出位置',
      defaultPath: `${packName.value}-${packVersion.value}.${meta.extension}`,
      filters: [{ name: `${meta.label} 整合包`, extensions: [meta.extension] }],
    })
    if (!savePath) { toastInfo('已取消导出'); return }

    // 非联网格式强制关闭 checkHostedAssets（避免无意义的联网请求）
    const finalCheckHostedAssets = supportsOnlineCheck.value && checkHostedAssets.value

    exporting.value = true
    resetProgress()
    try {
      const result = await exportModpack({
        versionId: selectedId.value,
        packName: packName.value,
        packVersion: packVersion.value,
        options: exportOptions.value,
        checkHostedAssets: finalCheckHostedAssets,
        modrinthUploadMode: modrinthUploadMode.value,
        configPackPath: savePath,
        format: exportFormat.value,
      })
      if (result.success) {
        const onlineInfo = finalCheckHostedAssets
          ? `，${result.modCount} 个 mod 联网下载`
          : ''
        toastSuccess(`导出成功：${result.fileCount} 个文件${onlineInfo}`)
        toastInfo(`文件位置：${result.filePath}`)
      } else {
        toastError('导出失败')
      }
    } catch (e) {
      toastError('导出失败：' + String(e))
    } finally {
      exporting.value = false
      // 保留进度状态显示完成/失败，3秒后重置
      setTimeout(() => resetProgress(), 3000)
    }
  }

  onMounted(() => {
    start()
  })

  return {
    // 状态
    loading,
    exporting,
    exportOptions,
    packName,
    packVersion,
    exportFormat,
    checkHostedAssets,
    modrinthUploadMode,
    // 进度状态
    exportProgress,
    exportStage,
    exportMessage,
    // 计算属性
    visibleOptions,
    topLevelOptions,
    currentFormatMeta,
    supportsOnlineCheck,
    formatOptions,
    // 方法
    getChildren,
    loadOptions,
    toggleOption,
    handleSaveConfig,
    handleLoadConfig,
    handleExport,
  }
}
