import { invokeCommand } from '@/lib/api'

export const skillService = {
  deleteManagedSkill(skillId: string): Promise<void> {
    return invokeCommand('delete_managed_skill', { skill_id: skillId })
  },

  setSkillTags(skillId: string, tagIds: number[]): Promise<void> {
    return invokeCommand('set_skill_tags', { skill_id: skillId, tag_ids: tagIds })
  },

  setSkillEnabled(skillId: string, enabled: boolean): Promise<void> {
    return invokeCommand('set_skill_enabled', { skill_id: skillId, enabled })
  },

  deleteManagedSkills(skillIds: string[]): Promise<{ removed: number }> {
    return invokeCommand('delete_managed_skills', { skill_ids: skillIds })
  },

  bulkSyncSkills(skillIds: string[]): Promise<{ synced: number; skipped: number; errors: string[] }> {
    return invokeCommand('bulk_sync_skills', { skill_ids: skillIds })
  },

  bulkSetSkillTags(skillIds: string[], tagIds: number[]): Promise<{ updated: number }> {
    return invokeCommand('bulk_set_skill_tags', { skill_ids: skillIds, tag_ids: tagIds })
  },
}
