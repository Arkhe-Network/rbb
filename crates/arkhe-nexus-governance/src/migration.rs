//! Guia de migração do Safe Core para GovernanceGuard.
/// Documentação de migração para desenvolvedores do NEXUS.
pub struct MigrationGuide;
/// Lista de stubs do NEXUS que precisam ser substituídos.
pub const STUBS_TO_REPLACE: &[&str] = &[
    "SafeCoreGuard::execute",
    "SafeCoreGuard::update_kernel",
    "SafeCoreGuard::modify_capsule",
    "SafeCoreGuard::update_compliance",
    "NEXUS::admin_action",
    "NEXUS::privileged_operation",
];
/// Verifica se um módulo ainda contém stubs não migrados.
pub fn check_migration_status(code: &str) -> Vec<&'static str> {
    STUBS_TO_REPLACE
        .iter()
        .filter(|stub| code.contains(*stub))
        .copied()
        .collect()
}
