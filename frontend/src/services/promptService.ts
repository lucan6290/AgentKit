import type { Prompt, PromptFileLink } from '@/features/prompts/types'
import { invokeCommand } from '@/lib/api'

export const promptService = {
  listPrompts(): Promise<Prompt[]> {
    return invokeCommand('list_prompts')
  },

  createPrompt(name: string, content: string): Promise<Prompt> {
    return invokeCommand('create_prompt', { name, content })
  },

  updatePrompt(id: string, name: string, content: string): Promise<Prompt> {
    return invokeCommand('update_prompt', { id, name, content })
  },

  duplicatePrompt(id: string, name?: string): Promise<Prompt> {
    return invokeCommand('duplicate_prompt', { id, name: name ?? null })
  },

  deletePrompt(id: string): Promise<void> {
    return invokeCommand('delete_prompt', { id })
  },

  scanToolPromptFiles(): Promise<{ scanned: number; created: number; updated: number }> {
    return invokeCommand('scan_tool_prompt_files')
  },

  importPromptFile(filePath: string, name?: string): Promise<Prompt> {
    return invokeCommand('import_prompt_file', { file_path: filePath, name: name ?? null })
  },

  createPromptFileLink(
    promptId: string,
    filePath: string,
    writeBackEnabled: boolean,
  ): Promise<PromptFileLink> {
    return invokeCommand('create_prompt_file_link', {
      prompt_id: promptId,
      file_path: filePath,
      write_back_enabled: writeBackEnabled,
    })
  },

  unlinkPromptFile(linkId: string): Promise<void> {
    return invokeCommand('unlink_prompt_file', { link_id: linkId })
  },

  refreshPromptFileLink(linkId: string): Promise<Prompt> {
    return invokeCommand('refresh_prompt_file_link', { link_id: linkId })
  },

  writePromptToFile(linkId: string, force = false): Promise<PromptFileLink> {
    return invokeCommand('write_prompt_to_file', { link_id: linkId, force })
  },
}
