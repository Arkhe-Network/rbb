# 📚 Safe-Core Academic — Documentação para CAPES/RNP

## Resumo Executivo

O **Safe-Core Academic** é uma camada de interoperabilidade semântica que conecta os sistemas acadêmicos brasileiros (SIGAA, SIPAC, Sucupira) ao ecossistema Safe-Core, permitindo:

1. **Integração automatizada** com a Plataforma Sucupira
2. **Validação de dados** com regras CAPES
3. **Auditoria imutável** via Merkle Tree
4. **Pseudonimização de dados sensíveis** (LGPD compliant)
5. **Roteamento inteligente** de dados acadêmicos

---

## 📊 Mapeamento de Sistemas

| Sistema | Função | Status | Integração |
|---------|--------|--------|------------|
| **SIGAA** | Gestão acadêmica | ✅ Implementado | Adapter SIGAA UnB |
| **SIPAC** | Gestão administrativa | 🔄 Em desenvolvimento | - |
| **Sucupira** | Coleta CAPES | ✅ Implementado | Adapter Sucupira |
| **CAFe** | Identidade federada | ✅ Implementado | SAML Parser |
| **GT BAITA** | Barramento | 🔄 Planejado | - |

---

## 🔐 Segurança e LGPD

- **Pseudonimização imediata**: CPF/matrícula hasheados com Blake3
- **Separação de dados**: Safe-Core nunca vê dados em claro
- **Audit trail**: Merkle Tree com assinatura criptográfica
- **Controle de acesso**: RBAC (gestor, profissional, auditor)

---

## 📋 Casos de Uso

### 1. Integração com Sucupira

**Antes:** Coleta manual, retrabalho para IES

**Depois:**
1. IES extrai dados do SIGAA/SIPAC
2. Adapter traduz para AcademicRecord
3. Safe-Core valida regras CAPES
4. Dados são enviados automaticamente para Sucupira
5. Toda operação é auditada na Merkle Tree

### 2. Validação de Programas CAPES

**Antes:** Verificação manual

**Depois:**
1. Adapter valida programa contra lista CAPES
2. Verifica conceito (≥ 3)
3. Registra resultado na Merkle Tree
4. Gera relatório de conformidade

---

## 🚀 Instalação

```bash
# Clone o repositório
git clone https://github.com/safe-core/academic.git
cd academic

# Compile o crate
cargo build --release -p safe-core-academic

# Execute os testes
cargo test -p safe-core-academic
```

---

## 📄 Exemplo de Uso

```rust
use safe_core_academic::{AcademicAdapter, sigaa::SigaaAdapter};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SigaaAdapter::new("UnB");

    let raw_data = json!({
        "idPessoa": "12345678",
        "idCurso": "PPGEC",
        "idNivel": "D",
        "idSituacao": "ATIVO"
    });

    let record = adapter.translate(&raw_data).await?;
    println!("✅ Registro processado: {}", record.course_program);

    Ok(())
}
```

---

## 📞 Contato

- **Email:** academic@safe-core.io
- **Repositório:** https://github.com/safe-core/academic
- **Documentação:** https://docs.safe-core.io/academic
