//! Testes de integração para arkhe-desci

use arkhe_desci::*;

#[test]
fn test_full_plugin_validation_flow() {
    let validator = plugin_governance::PluginValidator::default();

    let manifest = plugin_governance::PluginManifest {
        id: "bioinfo-tools".to_string(),
        name: "Bioinformatics Tools".to_string(),
        version: "2.1.0".to_string(),
        source: "https://github.com/example/bioinfo-tools".to_string(),
        signature: Some("sha256:abcdef123456".to_string()),
        install_script: "apt install -y samtools bcftools".to_string(),
        requested_permissions: vec!["network".to_string(), "fs_read".to_string()],
        dependencies: vec![],
        checksum: Some("sha256:fedcba654321".to_string()),
    };

    let result = validator.validate(&manifest).unwrap();
    assert!(result.passed);
    assert!(result.checks.iter().all(|c| c.passed));
}

#[test]
fn test_full_guardrail_flow() {
    let guardrails = assistant_guardrails::DeSciAssistantGuardrails::new();
    let ctx = assistant_guardrails::AssistantContext::default();

    // Mensagem científica com PII
    let (processed, check) = guardrails
        .check_message(
            "Analyze the BRCA1 sequence and email results to researcher@uni.edu",
            &ctx,
        )
        .unwrap();

    assert!(check.safe);
    assert!(processed.contains("[EMAIL_REDACTED]"));
    assert!(!processed.contains("researcher@uni.edu"));
}

#[test]
fn test_full_workflow_trace_flow() {
    let mut trace = workflow_traceability::ScientificWorkflowTrace::new(
        "variant-calling",
        workflow_traceability::WorkflowType::Nextflow,
    );

    // Step 1: Download
    let mut s1 = workflow_traceability::WorkflowStep::new("download", "Download Reference", "wget")
        .with_parameters(serde_json::json!({"url": "https://example.com/ref.fa"}));
    s1.start();
    s1.complete(vec!["ref.fa".to_string()]);
    trace.add_step(s1).unwrap();

    // Step 2: Index
    let mut s2 = workflow_traceability::WorkflowStep::new("index", "Index Reference", "bwa")
        .with_parameters(serde_json::json!({"algorithm": "bwtsw"}))
        .with_inputs(vec!["ref.fa".to_string()]);
    s2.start();
    s2.complete(vec!["ref.fa.bwt".to_string(), "ref.fa.sa".to_string()]);
    trace.add_step(s2).unwrap();

    // Step 3: Align
    let mut s3 = workflow_traceability::WorkflowStep::new("align", "Align Reads", "bwa-mem")
        .with_inputs(vec!["ref.fa".to_string(), "reads.fq".to_string()]);
    s3.start();
    s3.complete(vec!["aligned.sam".to_string()]);
    trace.add_step(s3).unwrap();

    // Verificar
    assert_eq!(trace.total_count(), 3);
    assert_eq!(trace.completed_count(), 3);
    assert!(trace.verify());

    // Tamper e verificar falha
    trace.steps[1].name = "TAMPERED".to_string();
    assert!(!trace.verify());
}

#[test]
fn test_end_to_end_scientific_workflow() {
    // 1. Validar plugin de bioinformática
    let validator = plugin_governance::PluginValidator::default();
    let manifest = plugin_governance::PluginManifest {
        id: "variant-pipeline".to_string(),
        name: "Variant Calling Pipeline".to_string(),
        version: "1.0.0".to_string(),
        source: "https://github.com/example/variant-pipeline".to_string(),
        signature: Some("sig".to_string()),
        install_script: "apt install -y bwa samtools bcftools".to_string(),
        requested_permissions: vec!["network".to_string()],
        dependencies: vec![],
        checksum: None,
    };
    let plugin_result = validator.validate(&manifest).unwrap();
    assert!(plugin_result.passed);

    // 2. Verificar query do usuário com guardrails
    let guardrails = assistant_guardrails::DeSciAssistantGuardrails::new();
    let ctx = assistant_guardrails::AssistantContext::default();
    let (safe_query, guard_result) = guardrails
        .check_message("Run variant calling on sample BRCA1_001 with bwa-mem", &ctx)
        .unwrap();
    assert!(guard_result.safe);
    assert_eq!(
        safe_query,
        "Run variant calling on sample BRCA1_001 with bwa-mem"
    );

    // 3. Rastrear workflow
    let mut trace = workflow_traceability::ScientificWorkflowTrace::new(
        "BRCA1_variant_calling",
        workflow_traceability::WorkflowType::Nextflow,
    )
    .with_metadata("sample_id", "BRCA1_001")
    .with_metadata("plugin_id", "variant-pipeline");

    let mut step = workflow_traceability::WorkflowStep::new("vc-1", "Variant Call", "bcftools")
        .with_parameters(serde_json::json!({
            "sample": "BRCA1_001",
            "tool": "bwa-mem",
            "reference": "hg38"
        }));
    step.start();
    step.complete(vec!["BRCA1_001.vcf.gz".to_string()]);
    trace.add_step(step).unwrap();

    assert!(trace.verify());

    // 4. Preparar metadados para publicação
    let metadata = publishing::DatasetMetadata {
        name: "BRCA1_001 Variants".to_string(),
        description: "Variant calls from BRCA1 gene".to_string(),
        format: "vcf.gz".to_string(),
        version: "1.0.0".to_string(),
        author_did: "did:arkhe:researcher-001".to_string(),
        license: "CC-BY-4.0".to_string(),
        tags: vec!["genomics".into(), "brca1".into()],
        created_at: "2026-07-01T12:00:00Z".to_string(),
        checksum_sha256: "abc123".to_string(),
    };

    // Serializar para JSON (em produção, publicar via IPFS)
    let json = serde_json::to_string_pretty(&metadata).unwrap();
    assert!(json.contains("BRCA1_001"));
}
