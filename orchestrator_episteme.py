import json
import logging
from discourse_detector_cientifico import DiscourseDetectorCientifico
from expand_ontology import generate_ontology
from generate_zk_proof import ZKProofGenerator

class EpistemicOrchestrator:
    def __init__(self):
        logging.basicConfig(level=logging.INFO)
        self.logger = logging.getLogger("EpistemicOrchestrator")

    def run_pipeline(self):
        self.logger.info("1. Expanding Ontology with 100+ concepts per domain...")
        generate_ontology()

        self.logger.info("2. Generating ZK-Proof for Ontological Consistency...")
        generator = ZKProofGenerator()
        proof = generator.generate_proof_for_ontology("episteme_expanded.json")
        with open('zk_proof_ontologia.json', 'w', encoding='utf-8') as f:
            json.dump(proof, f, indent=2)
        self.logger.info(f"ZK-Proof saved. Root: {proof['public_signals']['ontology_root']}")

        self.logger.info("3. Integrating with DiscourseDetectorCientifico for Real-Time Analysis...")
        detector = DiscourseDetectorCientifico(ontology_file="episteme_expanded.json")

        sample_abstract = """
        This paper discusses the role of Quantum Entanglement in the Standard Model.
        We use Deep Learning and a Transformer Model to analyze the Wave Function Collapse.
        Our findings show evidence that challenges the Null Hypothesis with high Statistical Power.
        However, the Placebo Effect in the Clinical Trial must be considered to avoid P-hacking
        and ensure Reproducibility. Epistemic Justice demands acknowledging Traditional Ecological Knowledge.
        """

        self.logger.info("Analyzing sample academic abstract...")
        report = detector.analyze_discourse(sample_abstract)

        with open('discourse_analysis_report.json', 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2)

        self.logger.info("Pipeline completed successfully.")
        return report

if __name__ == "__main__":
    orchestrator = EpistemicOrchestrator()
    orchestrator.run_pipeline()
