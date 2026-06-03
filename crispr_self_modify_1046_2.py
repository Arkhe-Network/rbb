from dataclasses import dataclass

@dataclass
class CRISPRGuide:
    target_site: str
    repair_template: str
