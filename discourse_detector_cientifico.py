import json
import logging

class DiscourseDetectorCientifico:
    """
    Detector de Discurso Científico para análise em tempo real da produção acadêmica,
    integrado com a ontologia epistêmica do Cathedral Arkhe.
    """
    def __init__(self, ontology_file="episteme_full.json"):
        self.ontology_file = ontology_file
        self.ontology = self._load_ontology()
        logging.info(f"DiscourseDetectorCientifico initialized with ontology from {ontology_file}")

    def _load_ontology(self):
        try:
            with open(self.ontology_file, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            logging.error(f"Error loading ontology: {e}")
            return {}

    def analyze_discourse(self, text, metadata=None):
        """
        Analisa um texto científico e retorna um relatório de discurso baseado na ontologia epistêmica.
        """
        if not text:
            return {"error": "No text provided for analysis."}

        report = {
            "text_length": len(text),
            "detected_concepts": [],
            "discourse_profile": {
                "analyst": 0,
                "university": 0,
                "master": 0,
                "hysteric": 0,
                "capitalist": 0
            },
            "epistemic_score": 0.0,
            "metadata": metadata or {}
        }

        # Simple string matching for demonstration purposes.
        # In a real scenario, this would use NLP and entity resolution.
        nodes = self.ontology.get("nodes", {})
        for node_id, node_data in nodes.items():
            label = node_data.get("label", "").lower()
            if label and label in text.lower():
                report["detected_concepts"].append({
                    "id": node_id,
                    "label": node_data.get("label"),
                    "category": node_data.get("category"),
                    "discourse": node_data.get("discourse"),
                    "status": node_data.get("status")
                })

                # Update discourse profile
                discourse_type = node_data.get("discourse")
                if discourse_type in report["discourse_profile"]:
                    report["discourse_profile"][discourse_type] += 1

        # Calculate a basic epistemic score based on detected concepts
        total_concepts = len(report["detected_concepts"])
        if total_concepts > 0:
            canonized_count = sum(1 for c in report["detected_concepts"] if c["status"] == "canonized")
            report["epistemic_score"] = canonized_count / total_concepts

        return report

if __name__ == "__main__":
    detector = DiscourseDetectorCientifico()
    sample_text = "The study provides evidence for gene expression and its causality. We also observed some p-hacking, which affects reproducibility and p-value."
    result = detector.analyze_discourse(sample_text)
    print(json.dumps(result, indent=2))
