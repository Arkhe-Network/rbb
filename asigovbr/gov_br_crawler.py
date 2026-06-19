import hashlib
import time
from typing import Dict, Any, List

class GovBrCrawler:
    """
    Crawler module for indexing sitemap.gov.br and storing in WormGraph (Arweave).
    """

    def __init__(self, start_url: str = "https://www.gov.br/pt-br/sitemap"):
        self.start_url = start_url
        self.wormgraph_mock_db: Dict[str, Any] = {}

    def crawl_sitemap(self, url: str) -> List[Dict[str, str]]:
        """
        Simulates crawling a gov.br sitemap and downloading documents.
        Returns a list of raw documents.
        """
        print(f"[Crawler] Crawling {url}...")
        # Mocking downloaded documents
        docs = [
            {
                "url": "https://www.gov.br/planalto/pt-br/constituicao",
                "title": "Constituição Federal",
                "type": "LEGISLACAO",
                "content": "Art. 1º A República Federativa do Brasil, formada pela união indissolúvel dos Estados e Municípios e do Distrito Federal, constitui-se em Estado Democrático de Direito e tem como fundamentos:\nI - a soberania;\nII - a cidadania;\nParágrafo único. Todo o poder emana do povo, que o exerce por meio de representantes eleitos ou diretamente, nos termos desta Constituição."
            },
            {
                "url": "https://www.gov.br/saude/pt-br/portaria-123",
                "title": "Portaria Ministério da Saúde 123/2026",
                "type": "PORTARIA",
                "content": "Art. 1º Fica estabelecido o novo repasse de recursos para a atenção primária.\nParágrafo único. Os recursos devem ser aplicados em 30 dias."
            }
        ]
        return docs

    def store_in_wormgraph(self, document: Dict[str, str]) -> str:
        """
        Simulates immutable storage in WormGraph (Arweave) and returns the transaction hash.
        """
        doc_string = f"{document['url']}|{document['content']}|{time.time()}"
        tx_hash = hashlib.sha256(doc_string.encode('utf-8')).hexdigest()

        # Add tags as required by the architecture
        stored_entry = {
            "tx_hash": tx_hash,
            "document": document,
            "tags": {
                "orgao": document.get("url", "").split("/")[3] if len(document.get("url", "").split("/")) > 3 else "unknown",
                "tipo": document.get("type", "UNKNOWN"),
                "data_coleta": time.time()
            }
        }

        self.wormgraph_mock_db[tx_hash] = stored_entry
        print(f"[WormGraph] Document '{document['title']}' stored immutably with hash {tx_hash[:16]}...")
        return tx_hash

    def get_document(self, tx_hash: str) -> Dict[str, Any]:
        """
        Retrieves a document from the WormGraph mock database.
        """
        return self.wormgraph_mock_db.get(tx_hash, {})
