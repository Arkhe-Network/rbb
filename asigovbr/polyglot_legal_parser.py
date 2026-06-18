from enum import Enum
import re
from typing import List, Optional, Dict, Any

class NodeKind(Enum):
    ROOT = "ROOT"
    ARTIGO = "ARTIGO"
    PARAGRAFO = "PARAGRAFO"
    INCISO = "INCISO"
    ALINEA = "ALINEA"
    TEXTO = "TEXTO"

class ASTNode:
    def __init__(self, kind: NodeKind, text: str, identifier: str = ""):
        self.kind = kind
        self.text = text
        self.identifier = identifier
        self.children: List['ASTNode'] = []

    def add_child(self, child: 'ASTNode'):
        self.children.append(child)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "kind": self.kind.value,
            "identifier": self.identifier,
            "text": self.text.strip(),
            "children": [c.to_dict() for c in self.children]
        }

class PolyglotLegalParser:
    """
    Parser adaptado para extrair estrutura hierárquica de textos jurídicos brasileiros.
    Transforma texto puro em uma AST (Abstract Syntax Tree).
    """

    def __init__(self):
        # Expressões regulares para capturar os elementos jurídicos
        self.re_artigo = re.compile(r'^(Art\.\s*\d+[º\.]?)(.*)', re.IGNORECASE | re.MULTILINE)
        self.re_paragrafo = re.compile(r'^((?:§\s*\d+[º\.]?)|Parágrafo\s+único)(.*)', re.IGNORECASE | re.MULTILINE)
        self.re_inciso = re.compile(r'^([A-Z]+)\s*-(.*)', re.MULTILINE)
        self.re_alinea = re.compile(r'^([a-z])\)(.*)', re.MULTILINE)

    def parse(self, text: str) -> ASTNode:
        root = ASTNode(NodeKind.ROOT, "Document Root")
        lines = text.split('\n')

        current_artigo: Optional[ASTNode] = None
        current_paragrafo: Optional[ASTNode] = None
        current_inciso: Optional[ASTNode] = None

        for line in lines:
            line = line.strip()
            if not line:
                continue

            # Match Artigo
            match_art = self.re_artigo.match(line)
            if match_art:
                ident, content = match_art.groups()
                current_artigo = ASTNode(NodeKind.ARTIGO, content, ident.strip())
                root.add_child(current_artigo)
                current_paragrafo = None
                current_inciso = None
                continue

            # Match Parágrafo
            match_par = self.re_paragrafo.match(line)
            if match_par:
                ident, content = match_par.groups()
                current_paragrafo = ASTNode(NodeKind.PARAGRAFO, content, ident.strip())
                if current_artigo:
                    current_artigo.add_child(current_paragrafo)
                else:
                    root.add_child(current_paragrafo)
                current_inciso = None
                continue

            # Match Inciso
            match_inc = self.re_inciso.match(line)
            if match_inc:
                ident, content = match_inc.groups()
                current_inciso = ASTNode(NodeKind.INCISO, content, ident.strip())

                if current_paragrafo:
                    current_paragrafo.add_child(current_inciso)
                elif current_artigo:
                    current_artigo.add_child(current_inciso)
                else:
                    root.add_child(current_inciso)
                continue

            # Match Alínea
            match_ali = self.re_alinea.match(line)
            if match_ali:
                ident, content = match_ali.groups()
                alinea_node = ASTNode(NodeKind.ALINEA, content, ident.strip())
                if current_inciso:
                    current_inciso.add_child(alinea_node)
                elif current_paragrafo:
                    current_paragrafo.add_child(alinea_node)
                elif current_artigo:
                    current_artigo.add_child(alinea_node)
                else:
                    root.add_child(alinea_node)
                continue

            # Se for apenas texto sem formatação clara, atrela ao nó atual mais específico
            texto_node = ASTNode(NodeKind.TEXTO, line)
            if current_inciso:
                current_inciso.add_child(texto_node)
            elif current_paragrafo:
                current_paragrafo.add_child(texto_node)
            elif current_artigo:
                current_artigo.add_child(texto_node)
            else:
                root.add_child(texto_node)

        return root
