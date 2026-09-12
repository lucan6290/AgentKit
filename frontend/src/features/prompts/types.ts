export type PromptFileLink = {
  id: string
  prompt_id: string
  file_path: string
  write_back_enabled: boolean
  content_hash: string | null
  exists_on_disk: boolean
  last_synced_at: number | null
  created_at: number
  updated_at: number
}

export type Prompt = {
  id: string
  name: string
  content: string
  file_links: PromptFileLink[]
  created_at: number
  updated_at: number
}
