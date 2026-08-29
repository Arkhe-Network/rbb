#!/usr/bin/env python3
"""
Catedral OS v10.9 — Orquestrador Python (Substrato 212 Standalone v5.1)
Integra: substrate_212.py (Canônico) + KeyManager RSA-4096 + Decimal ANATEL + Retries
"""

import os
import re
import json
import time
import threading
import http.server
import socketserver
import ssl
import hashlib
import logging
from typing import Dict, List, Any, Optional
from dataclasses import asdict

from substrate_212 import CertificateGateway, is_frequency_allowed, get_restriction_reason
from network_orchestrator import NetworkOrchestrator, NetworkConfig

logging.basicConfig(level=logging.INFO, format='%(asctime)s [Cathedral] %(levelname)s: %(message)s')
logger = logging.getLogger('cathedral.v109')

SSL_CERT_FILE = "server.crt"
SSL_KEY_FILE = "server.key"

class JWTManager:
    def __init__(self, gw: CertificateGateway): self.gw = gw
    def create_token(self, payload: Dict) -> str: return self.gw.issue_jwt(payload.get("user", "architect"))
    def verify_token(self, token: str) -> bool:
        try: self.gw.verify_jwt(token); return True
        except: return False

class ICCIDReader:
    @staticmethod
    def read_from_pysim(reader: Optional[str] = None) -> Optional[str]:
        import subprocess
        try:
            cmd = ["pySim-shell.py", "--json", "export"]
            if reader: cmd.extend(["-p", reader])
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            if result.returncode != 0: return None
            try:
                data = json.loads(result.stdout)
                if "profile_info" in data: return data["profile_info"].get("iccid")
                if "iccid" in data: return data["iccid"]
            except json.JSONDecodeError:
                import re
                match = re.search(r'ICCID["\s:]+([0-9]{18,22})', result.stdout)
                if match: return match.group(1)
            return None
        except FileNotFoundError: return None

class HumanizerEngine:
    """
    Substrato 267 — Humanizer Integration
    Implementa os 35 padrões para humanização de texto.
    """

    PATTERNS = {
        'inflated_importance': (r'marking a pivotal moment in the evolution of', r'was established in 1989 as part of a wider decentralization'),
        'name_dropping': (r'cited in NYT, BBC, FT, and The Hindu', r'Keep only useful, sourced context'),
        'shallow__ing_analysis': (r'(symbolizing|reflecting|showcasing)', r'Keep only what the source supports'),
        'sales_language': (r' nestled within the breathtaking', r' is a town in the Gonder region'),
        'vague_sources': (r'Experts believe it plays a crucial role', r'Name a real source or remove the claim'),
        'formulaic_challenges': (r'Despite challenges.*continues to thrive', r'Keep the facts and remove the sales pitch'),
        'overused_ai_words': (r'(actually|additionally|showcasing|testament|landscape)', r'also|needs|remain common'),
        'avoiding_is_are': (r'(serves as|features|boasts)', r'is|has'),
        'not_x_but_y': (r'It\'s not just .*, it\'s .*', r'State the point directly'),
        'forced_groups_of_three': (r'innovation, inspiration, and insights', r'Use the number of items the meaning needs'),
        'changing_names': (r'(protagonist|main character|hero)', r'Use one name'),
        'false_x_to_y_ranges': (r'from the Big Bang to dark matter', r'List the topics directly'),
        'passive_voice': (r'No configuration file needed', r'Name the actor when that helps'),
        'em_en_dashes': (r'institutions—not the people—yet this continues—', r'Cut them: periods, commas, colons'),
        'too_much_bold': (r'\*\*OKRs\*\*, \*\*KPIs\*\*, \*\*BMC\*\*', r'OKRs, KPIs, BMC'),
        'lists_with_bold_headings': (r'\*\*Performance:\*\* Performance improved', r'Use prose when a list adds no value'),
        'title_case_in_headings': (r'Strategic Negotiations And Partnerships', r'Strategic negotiations and partnerships'),
        'emojis': (r'🚀 Launch Phase: Key Insight:', r'Remove emojis'),
        'curly_quotes': (r'said “the project”', r'said "the project"'),
        'chatbot_text': (r'I hope this helps! Let me know if.*', r'Remove it'),
        'knowledge_limit_disclaimers': (r'While details are limited in available sources.*', r'State what is known or remove the claim'),
        'overly_agreeable_tone': (r'Great question! You\'re absolutely right!', r'Answer directly'),
        'filler_phrases': (r'In order to', r'To'),
        'too_many_qualifiers': (r'could potentially possibly', r'may'),
        'generic_positive_endings': (r'The future looks bright', r'End with a fact or a sourced plan'),
        'too_many_hyphenated_pairs': (r'cross-functional, data-driven, client-facing', r'Keep only the hyphens grammar needs'),
        'fake_deeper_truth': (r'At its core, what matters is.*', r'State the point directly'),
        'announcing_the_next_point': (r'Let\'s dive in', r'Start with the content'),
        'heading_repeated_below_itself': (r'## Performance.*Speed matters.', r'Let the heading do the work'),
        'writing_about_the_old_version': (r'This function was added to replace.*', r'Describe what it does now'),
        'forced_punchlines': (r'It had no preference. No prior. No nostalgia.', r'Use natural sentence lengths'),
        'formulaic_sayings': (r'Symmetry is the language of trust', r'State the specific claim'),
        'fake_candid_openings': (r'Honestly\? It depends.*', r'State the answer directly'),
        'answering_objections_no_one_raised': (r'This isn\'t mainly about prompt length.*', r'Remove the unsupported defense'),
        'rejecting_fake_alternatives': (r'A tempting option would be to.*, but', r'Remove the fake option'),
    }

    def __init__(self, prolog_core=None):
        self.prolog = prolog_core

    def detect_patterns(self, text: str) -> List[str]:
        """Detecta padrões de IA no texto."""
        found = []
        for name, (pattern, _) in self.PATTERNS.items():
            if re.search(pattern, text, re.IGNORECASE):
                found.append(name)
        return found

    def humanize(self, text: str) -> str:
        """Aplica humanização ao texto."""
        result = text
        for name, (pattern, replacement) in self.PATTERNS.items():
            result = re.sub(pattern, replacement, result, flags=re.IGNORECASE)
        return result

    def humanize_with_style(self, text: str, sample: str) -> str:
        """Humaniza seguindo um sample de estilo."""
        # Em produção: extrair métricas de estilo do sample
        return self.humanize(text)

class CathedralCore:
    def __init__(self, prolog_file: str = "agi_core.pl"):
        self.prolog = None
        self.cert_gateway = CertificateGateway()
        self.jwt_manager = JWTManager(self.cert_gateway)
        self.iccid_reader = ICCIDReader()
        try:
            from pyswip import Prolog
            self.prolog = Prolog()
            self.prolog.consult(prolog_file)
            list(self.prolog.query("agi_init"))
        except ImportError: pass
        self.humanizer = HumanizerEngine(self.prolog)

    def think(self, context: str) -> Dict[str, Any]:
        if self.prolog:
            safe = context.replace("'", "\\'").replace('"', '\\"')
            try:
                results = list(self.prolog.query(f"think('{safe}', Output, Status)"))
                if results: return {"output": str(results[0].get("Output", "")), "status": str(results[0].get("Status", "error"))}
            except: pass
        return {"output": "✅ Cognição estável (fallback)", "status": "success"}

class WormGraph:
    def __init__(self): self.ledger: List[Dict] = []; self._lock = threading.Lock()
    def commit(self, block: Dict) -> bool:
        with self._lock:
            block['index'] = len(self.ledger); block['timestamp'] = time.time()
            block['hash'] = hashlib.sha256(json.dumps(block, sort_keys=True).encode()).hexdigest()
            self.ledger.append(block); return True

class CathedralHandler(http.server.SimpleHTTPRequestHandler):
    core: CathedralCore = None; wormgraph: WormGraph = None; network: NetworkOrchestrator = None

    def _check_auth(self) -> bool:
        auth = self.headers.get('Authorization')
        if auth and auth.startswith('Bearer '): return self.core.jwt_manager.verify_token(auth.split(' ')[1])
        return False

    def do_GET(self):
        if self.path == '/api/login':
            token = self.core.jwt_manager.create_token({"user": "architect"})
            self._json_response({"token": token}); return
        if not self._check_auth(): self._json_response({"error": "Unauthorized"}, 401); return

        if self.path == '/api/health': self._json_response({
            "status": "online", "version": "10.9", "security": "HTTPS + JWT RS256 (RSA-4096)",
            "gateway": asdict(self.core.cert_gateway.get_status())
        })
        elif self.path.startswith('/api/ct_check?domain='):
            domain = self.path.split('=')[1]; logs = self.core.cert_gateway.check_ct_logs(domain)
            self._json_response(logs)
        elif self.path == '/' or self.path == '/index.html':
            self.path = '/clareira_fractal.html'; super().do_GET()
        else: super().do_GET()

    def do_POST(self):
        if not self._check_auth() and self.path != '/api/login': self._json_response({"error": "Unauthorized"}, 401); return

        if self.path == '/api/think':
            body = self.rfile.read(int(self.headers['Content-Length'])).decode()
            data = json.loads(body); self._json_response(self.core.think(data.get('input', '')))
        elif self.path == '/api/cert/generate':
            body = self.rfile.read(int(self.headers['Content-Length'])).decode()
            data = json.loads(body); cert_data = self.core.cert_gateway.generate_certificate(data.get("cn", "cathedral.local"))
            self.wormgraph.commit({"type": "cert_issuance", "cn": data.get("cn")})
            self._json_response(cert_data)
        elif self.path == '/api/humanize':
            body = self.rfile.read(int(self.headers['Content-Length'])).decode()
            data = json.loads(body)
            humanized_text = self.core.humanizer.humanize(data.get('text', ''))
            self._json_response({"humanized": humanized_text})
        else: self._json_response({"error": "Not found"}, 404)

    def _json_response(self, data: Any, code: int = 200):
        self.send_response(code); self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*'); self.end_headers()
        self.wfile.write(json.dumps(data, default=str).encode())

def main():
    prolog_file = "agi_core.pl"
    if not os.path.exists(prolog_file):
        with open(prolog_file, 'w') as f: f.write(":- module(cathedral_v109, [agi_init/0, think/3]).\nagi_init :- true.\nthink(I,O,S) :- S=success.\n")

    core = CathedralCore(prolog_file)
    wormgraph = WormGraph()
    network = NetworkOrchestrator(NetworkConfig()); network.start()

    CathedralHandler.core = core; CathedralHandler.wormgraph = wormgraph; CathedralHandler.network = network
    wormgraph.commit({"event": "cathedral_v109_init", "version": "10.9"})

    PORT = 8443
    if not os.path.exists(SSL_CERT_FILE) or not os.path.exists(SSL_KEY_FILE):
        os.system(f"openssl req -x509 -newkey rsa:2048 -keyout {SSL_KEY_FILE} -out {SSL_CERT_FILE} -days 365 -nodes -subj '/CN=localhost'")

    httpd = socketserver.TCPServer(("", PORT), CathedralHandler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(certfile=SSL_CERT_FILE, keyfile=SSL_KEY_FILE)
    httpd.socket = context.wrap_socket(httpd.socket, server_side=True)

    print(f"\n{'='*60}\n  🏛️ CATEDRAL OS v10.9 — Substrato 212 Standalone v5.1\n{'='*60}")
    print(f"  HTTPS:      https://localhost:{PORT}\n  Auth:       JWT RS256 (RSA-4096 Persistente)")
    print(f"  Gateway:    ANATEL Canônico + cryptography.__version__ Fixed\n{'='*60}\n")

    try: httpd.serve_forever()
    except KeyboardInterrupt: print("\nParando..."); network.shutdown()

if __name__ == "__main__": main()