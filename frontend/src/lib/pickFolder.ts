import { open } from '@tauri-apps/plugin-dialog'

/**
 * 打开系统原生文件夹选择对话框，返回用户选择的文件夹路径。
 * 用户取消时返回 null。
 */
export async function pickFolder(promptTitle: string): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false, title: promptTitle })
  if (typeof selected !== 'string') return null
  return selected
}

/**
 * 打开系统原生文件选择对话框，返回用户选择的文件路径。
 * 用户取消时返回 null。
 */
export async function pickFile(promptTitle: string): Promise<string | null> {
  const selected = await open({ directory: false, multiple: false, title: promptTitle })
  if (typeof selected !== 'string') return null
  return selected
}
