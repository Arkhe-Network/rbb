pub mod grill_me;
pub mod to_prd;
pub mod diagnose;
pub mod tdd;
pub mod improve_architecture;
pub mod triage;

use crate::skill::types::Skill;
use crate::skill::manager::SkillManager;

pub async fn register_all(skill_mgr: &mut SkillManager) -> Result<Vec<String>, String> {
    let skills = vec![
        grill_me::grill_me_skill(),
        to_prd::to_prd_skill(),
        diagnose::diagnose_skill(),
        tdd::tdd_skill(),
        improve_architecture::improve_architecture_skill(),
        triage::triage_skill(),
    ];

    let mut registered = Vec::new();
    for skill in skills {
        let _ = skill_mgr.save_skill(&skill)?;
        registered.push(skill.name);
    }

    Ok(registered)
}
