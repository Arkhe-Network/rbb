import json
import time
from polyglot_legal_parser import PolyglotLegalParser
from fast_brain_gov import FastBrainGov
from gov_br_crawler import GovBrCrawler

class ASIGovBrOrchestrator:
    """
    Orchestrates the ASI-GOV.BR pipeline:
    1. Crawls gov.br and stores in WormGraph
    2. Parses legal documents via Polyglot-Parser
    3. Analyzes policies via Fast Brain
    4. Simulates governance proposition on-chain
    """

    def __init__(self):
        self.crawler = GovBrCrawler()
        self.parser = PolyglotLegalParser()
        self.brain = FastBrainGov()

    def run_pipeline(self):
        print("="*60)
        print("🏛️ Iniciando Orquestrador ASI-GOV.BR")
        print("="*60)

        # 1. Coleta e Armazenamento
        print("\n[Fase 1] Coletando documentos governamentais...")
        docs = self.crawler.crawl_sitemap("https://www.gov.br/pt-br/sitemap")

        target_hash = None
        for doc in docs:
            tx_hash = self.crawler.store_in_wormgraph(doc)
            if doc['type'] == 'PORTARIA':  # Vamos focar na portaria para análise
                target_hash = tx_hash

        if not target_hash:
            print("Nenhum documento alvo encontrado.")
            return

        stored_doc = self.crawler.get_document(target_hash)
        raw_text = stored_doc['document']['content']

        print(f"\n[Fase 2] Processando documento ({stored_doc['document']['title']})...")

        # 2. Processamento: Polyglot-Parser
        ast = self.parser.parse(raw_text)
        print(f"AST Extraída com sucesso! Raiz tem {len(ast.children)} elementos principais.")

        # 3. Análise: Fast Brain
        print("\n[Fase 3] Análise do Fast Brain...")
        summary = self.brain.summarize_legal_text(raw_text)
        print(f"Resumo Executivo:\n{summary}")

        impact = self.brain.analyze_policy_impact(ast)
        print(f"\nAnálise de Impacto:")
        print(json.dumps(impact, indent=2, ensure_ascii=False))

        contradictions = self.brain.detect_contradictions(ast)
        if contradictions:
            print(f"\n🚨 Contradições Detectadas: {len(contradictions)}")
            for c in contradictions:
                print(f"  - {c['element']}: {c['issue']}")
        else:
            print("\n✅ Nenhuma contradição detectada.")

        # 4. Proposição e Governança
        print("\n[Fase 4] Governança e Participação Cidadã...")
        demand = "Aumentar prazo de aplicação dos recursos para 90 dias devido a burocracias municipais."
        print(f"Demanda Popular Captada: '{demand}'")

        amendment = self.brain.generate_amendment_proposal(ast, demand)
        print(f"\nProposta Gerada Automaticamente:\n{amendment}")

        print("\nSubmetendo Proposta ao Contrato Inteligente (Simulação CatedralGovernance.sol)...")
        proposal_id = 101 # Simulated ID
        print(f"Proposta #{proposal_id} criada on-chain com hash do WormGraph {target_hash[:16]}")
        print("Aguardando votação popular com peso regional e veto do Conselho de Especialistas...")

        time.sleep(1)
        print("\n✅ Pipeline ASI-GOV.BR executado com sucesso.")

if __name__ == "__main__":
    orchestrator = ASIGovBrOrchestrator()
    orchestrator.run_pipeline()
