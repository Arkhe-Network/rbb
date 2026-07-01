//! Identity Bridge — Converte identidades acadêmicas (SAML/OIDC) para TenantId do Safe-Core
//!
//! O CAFe emite SAML Assertions; o GT BAITA emite OIDC tokens.
//! O Safe-Core usa TenantId via header. Este módulo faz a ponte.
//!
//! # Exemplo de SAML Assertion do CAFe
//!
//! ```xml
//! <saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
//!   <saml:Attribute Name="eduPersonPrincipalName">
//!     <saml:AttributeValue>joao.silva@unb.br</saml:AttributeValue>
//!   </saml:Attribute>
//!   <saml:Attribute Name="eduPersonAffiliation">
//!     <saml:AttributeValue>student</saml:AttributeValue>
//!   </saml:Attribute>
//!   <saml:Attribute Name="eduPersonScopedAffiliation">
//!     <saml:AttributeValue>student@unb.br</saml:AttributeValue>
//!   </saml:Attribute>
//! </saml:Assertion>
//! ```

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Atributo SAML ausente: {0}")]
    MissingAttribute(String),

    #[error("Erro de parsing XML: {0}")]
    XmlParseError(String),

    #[error("Papel acadêmico inválido: {0}")]
    InvalidRole(String),

    #[error("Assinatura SAML inválida")]
    InvalidSignature,
}

/// Claims extraídas do SAML Assertion do CAFe ou do OIDC Token do BAITA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicIdentity {
    /// Hash da chave pública da IES no CAFe (identificador soberano)
    pub institution_hash: String,

    /// CPF/ePPN do usuário hasheado (Safe-Core nunca vê o CPF em claro)
    pub user_hash: String,

    /// Papel extraído do atributo SAML/OIDC
    pub role: AcademicRole,

    /// Affiliation (ex: "membro:engenharia@ufsc.br")
    pub affiliation: String,

    /// Email institucional (opcional, para notificações)
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AcademicRole {
    Discente,
    Docente,
    Tecnico,
    Gestor,
    Pesquisador,
}

/// Extrai atributos de uma SAML Assertion usando quick-xml
pub struct SamlParser;

impl SamlParser {
    /// Extrai as claims de um token SAML Assertion (XML).
    /// Usa `quick-xml` para parsear o XML corretamente.
    pub fn from_saml_assertion(saml_xml: &str) -> Result<AcademicIdentity, IdentityError> {
        let mut reader = Reader::from_str(saml_xml);
        reader.trim_text(true);

        let mut in_attribute_value = false;
        let mut current_attr_name = String::new();
        let mut attr_values = Vec::new();

        let mut eppn = None;
        let mut affiliation = None;
        let mut scoped_affiliation = None;
        let mut issuer: Option<String> = None;

        let mut in_issuer = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = e.name();
                    let name_str = String::from_utf8_lossy(name.as_ref()).to_string();

                    if name_str == "Attribute" || name_str == "saml:Attribute" {
                        // Buscar o atributo Name
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Name" {
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                current_attr_name = value;
                                attr_values.clear();
                            }
                        }
                    }

                    if name_str == "AttributeValue" || name_str == "saml:AttributeValue" {
                        in_attribute_value = true;
                    }

                    if name_str == "Issuer" || name_str == "saml:Issuer" {
                        in_issuer = true;
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_attribute_value && !text.is_empty() {
                        attr_values.push(text.clone());
                    }
                    if in_issuer && !text.is_empty() {
                        issuer = Some(text);
                    }
                }
                Ok(Event::End(e)) => {
                    let name = e.name();
                    let name_str = String::from_utf8_lossy(name.as_ref()).to_string();

                    if name_str == "Attribute" || name_str == "saml:Attribute" {
                        // Final do atributo: processar valor
                        if let Some(value) = attr_values.first() {
                            match current_attr_name.as_str() {
                                "eduPersonPrincipalName" => {
                                    eppn = Some(value.clone());
                                }
                                "eduPersonAffiliation" => {
                                    affiliation = Some(value.clone());
                                }
                                "eduPersonScopedAffiliation" => {
                                    scoped_affiliation = Some(value.clone());
                                }
                                _ => {}
                            }
                        }
                        current_attr_name.clear();
                    }

                    if name_str == "AttributeValue" || name_str == "saml:AttributeValue" {
                        in_attribute_value = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IdentityError::XmlParseError(e.to_string()));
                }
                _ => {}
            }
            buf.clear();
        }

        // Validar campos obrigatórios
        let user_id =
            eppn.ok_or_else(|| IdentityError::MissingAttribute("eduPersonPrincipalName".into()))?;
        let aff = affiliation
            .ok_or_else(|| IdentityError::MissingAttribute("eduPersonAffiliation".into()))?;

        // Determinar papel
        let role = Self::parse_role(&aff)?;

        // Hash do user_id com blake3
        let user_hash = {
            let hash = blake3::hash(user_id.as_bytes());
            hex::encode(hash.as_bytes())
        };

        // Hash da instituição (baseado no issuer ou no domínio do ePPN)
        let institution_hash = if let Some(iss) = issuer {
            let hash = blake3::hash(iss.as_bytes());
            hex::encode(hash.as_bytes())
        } else if let Some(scoped) = &scoped_affiliation {
            // Extrair domínio de student@unb.br
            if let Some(domain) = scoped.split('@').next_back() {
                let hash = blake3::hash(domain.as_bytes());
                hex::encode(hash.as_bytes())
            } else {
                // Fallback: hash do ePPN
                let hash = blake3::hash(user_id.as_bytes());
                hex::encode(hash.as_bytes())
            }
        } else {
            // Fallback: hash do ePPN
            let hash = blake3::hash(user_id.as_bytes());
            hex::encode(hash.as_bytes())
        };

        Ok(AcademicIdentity {
            institution_hash,
            user_hash,
            role,
            affiliation: scoped_affiliation.unwrap_or(aff),
            email: Some(user_id),
        })
    }

    /// Converte string de affiliation em AcademicRole
    pub fn parse_role(affiliation: &str) -> Result<AcademicRole, IdentityError> {
        match affiliation.to_lowercase().as_str() {
            "student" | "discente" | "aluno" | "graduate" | "undergraduate" => {
                Ok(AcademicRole::Discente)
            }
            "faculty" | "docente" | "professor" | "teacher" | "staff" | "employee" => {
                if affiliation.contains("professor")
                    || affiliation.contains("docente")
                    || affiliation.contains("faculty")
                {
                    Ok(AcademicRole::Docente)
                } else {
                    Ok(AcademicRole::Tecnico)
                }
            }
            "manager" | "gestor" | "coordenador" | "director" | "admin" => Ok(AcademicRole::Gestor),
            "researcher" | "pesquisador" | "postdoc" | "research" => Ok(AcademicRole::Pesquisador),
            _ => {
                // Tentar inferir por palavras-chave
                let lower = affiliation.to_lowercase();
                if lower.contains("student") || lower.contains("aluno") {
                    Ok(AcademicRole::Discente)
                } else if lower.contains("professor") || lower.contains("docente") {
                    Ok(AcademicRole::Docente)
                } else if lower.contains("admin") || lower.contains("gestor") {
                    Ok(AcademicRole::Gestor)
                } else {
                    Err(IdentityError::InvalidRole(affiliation.to_string()))
                }
            }
        }
    }

    /// Converte a identidade acadêmica no TenantId esperado pelos handlers do Safe-Core
    pub fn to_safe_core_tenant(identity: &AcademicIdentity) -> String {
        format!(
            "acad:{}:{}",
            identity.institution_hash, identity.affiliation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SAML: &str = r#"
        <saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
            <saml:Issuer>https://cafe.unb.br/idp</saml:Issuer>
            <saml:Attribute Name="eduPersonPrincipalName">
                <saml:AttributeValue>joao.silva@unb.br</saml:AttributeValue>
            </saml:Attribute>
            <saml:Attribute Name="eduPersonAffiliation">
                <saml:AttributeValue>student</saml:AttributeValue>
            </saml:Attribute>
            <saml:Attribute Name="eduPersonScopedAffiliation">
                <saml:AttributeValue>student@unb.br</saml:AttributeValue>
            </saml:Attribute>
        </saml:Assertion>
    "#;

    #[test]
    fn test_saml_parsing_with_quick_xml() {
        let identity = SamlParser::from_saml_assertion(SAMPLE_SAML).unwrap();

        assert_eq!(identity.role, AcademicRole::Discente);
        assert_eq!(identity.email, Some("joao.silva@unb.br".to_string()));
        assert_eq!(identity.affiliation, "student@unb.br");
        assert!(!identity.user_hash.is_empty());
        assert!(!identity.institution_hash.is_empty());
        assert_eq!(identity.user_hash.len(), 64); // Blake3 hash
    }

    #[test]
    fn test_parse_role() {
        assert_eq!(
            SamlParser::parse_role("student").unwrap(),
            AcademicRole::Discente
        );
        assert_eq!(
            SamlParser::parse_role("discente").unwrap(),
            AcademicRole::Discente
        );
        assert_eq!(
            SamlParser::parse_role("aluno").unwrap(),
            AcademicRole::Discente
        );
        assert_eq!(
            SamlParser::parse_role("faculty").unwrap(),
            AcademicRole::Docente
        );
        assert_eq!(
            SamlParser::parse_role("professor").unwrap(),
            AcademicRole::Docente
        );
        assert_eq!(
            SamlParser::parse_role("staff").unwrap(),
            AcademicRole::Tecnico
        );
        assert_eq!(
            SamlParser::parse_role("manager").unwrap(),
            AcademicRole::Gestor
        );
        assert_eq!(
            SamlParser::parse_role("coordenador").unwrap(),
            AcademicRole::Gestor
        );
        assert_eq!(
            SamlParser::parse_role("researcher").unwrap(),
            AcademicRole::Pesquisador
        );
        assert!(SamlParser::parse_role("unknown").is_err());
    }

    #[test]
    fn test_tenant_generation() {
        let identity = AcademicIdentity {
            institution_hash: "abc123def456".to_string(),
            user_hash: "def456abc123".to_string(),
            role: AcademicRole::Docente,
            affiliation: "engenharia@unb.br".to_string(),
            email: Some("joao.silva@unb.br".to_string()),
        };

        let tenant = SamlParser::to_safe_core_tenant(&identity);
        assert_eq!(tenant, "acad:abc123def456:engenharia@unb.br");
    }
}
