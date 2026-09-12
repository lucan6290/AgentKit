pub mod install;
pub mod maintenance;
pub mod managed_skills;
pub mod onboarding;
pub mod prompts;

pub use install::{
    install_local_skill, install_local_skill_from_selection, list_local_skills, InstallResult,
    LocalSkillCandidate, SkillFrontmatter,
};
pub use maintenance::{repair_sync_health, scan_sync_health, SyncHealthReport};
pub use onboarding::{build_onboarding_plan, OnboardingGroup, OnboardingPlan, OnboardingVariant};
pub use prompts::{
    create_prompt, create_prompt_file_link, delete_prompt, duplicate_prompt, import_prompt_file,
    list_prompts, refresh_prompt_file_link, unlink_prompt_file, update_prompt,
    write_prompt_to_file,
};
