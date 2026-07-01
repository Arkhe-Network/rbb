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
    };

    let result = validator.validate(&manifest).unwrap();
    assert!(result.passed);
    assert!(result.checks.iter().all(|c| c.passed));
}

#[test]
fn test_full_guardrail_flow() {
    let guardrails = assistant_guardrails::DeSciAssistantGuardrails::new();
    let ctx = assistant_guardrails::AssistantContext::default();

    let (processed, check) = guardrails.check_message(
        "Analyze the BRCA1 sequence and email results to researcher@uni.edu",
        &ctx,
    ).unwrap();

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

    let mut s1 = workflow_traceability::WorkflowStep::new("download", "Download Reference", "wget")
        .with_parameters(serde_json::json!({"url": "https://example.com/ref.fa"}));
    s1.start();
    s1.complete(vec!["ref.fa".to_string()]);
    trace.add_step(s1).unwrap();

    let mut s2 = workflow_traceability::WorkflowStep::new("index", "Index Reference", "bwa")
        .with_parameters(serde_json::json!({"algorithm": "bwtsw"}))
        .with_inputs(vec!["ref.fa".to_string()]);
    s2.start();
    s2.complete(vec!["ref.fa.bwt".to_string(), "ref.fa.sa".to_string()]);
    trace.add_step(s2).unwrap();

    let mut s3 = workflow_traceability::WorkflowStep::new("align", "Align Reads", "bwa-mem")
        .with_inputs(vec!["ref.fa".to_string(), "reads.fq".to_string()]);
    s3.start();
    s3.complete(vec!["aligned.sam".to_string()]);
    trace.add_step(s3).unwrap();

    assert_eq!(trace.total_count(), 3);
    assert_eq!(trace.completed_count(), 3);
    assert!(trace.verify());

    trace.steps[1].name = "TAMPERED".to_string();
    assert!(!trace.verify());
}
