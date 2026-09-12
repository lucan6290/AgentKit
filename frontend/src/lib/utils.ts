/** 将字节数转为人类可读的文件大小字符串 */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

/** 将 Windows 路径统一为反斜杠，仅用于界面展示。 */
export function formatDisplayPath(filePath: string): string {
  return navigator.userAgent.includes('Windows') ? filePath.replaceAll('/', '\\') : filePath
}
