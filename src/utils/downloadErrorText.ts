/**
 * 下载失败码 → 一句能看懂的话
 *
 * 码表与后端 `minecraft/download/error_code.rs` 一一对应。前端拿到非 0 的
 * `error_code` 时必须有话可说——只把状态改成失败、不给原因，用户只能干瞪眼。
 */

const ERROR_TEXTS: Record<number, string> = {
  1: '下载失败：所有下载源都没能取到数据，可以换个下载源再试',
  2: '下载的文件校验不通过，可能被网络中间件改动过，请重试',
  3: '写入磁盘失败：请检查剩余空间与目录权限',
  4: '下载已取消',
  5: '安装失败：文件已下载，但后续安装步骤出错',
}

/** 非 0 的失败码给一句人话；0 或未知码返回 null（未知码由调用方兜底） */
export function describeDownloadError(code: number): string | null {
  if (!code) return null
  return ERROR_TEXTS[code] ?? null
}
