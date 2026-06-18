import json
from typing import Dict, Any, Optional

def get_prompt(task_type: str, context: Dict[str, Any], code: Optional[str] = None,
               language: Optional[str] = None, target_language: Optional[str] = None) -> str:
    if task_type == "security_fix":
        return _security_fix_prompt(context, code, language)
    elif task_type == "code_translation":
        return _code_translation_prompt(context, code, language, target_language)
    elif task_type == "architecture_improvement":
        return _architecture_improvement_prompt(context)
    elif task_type == "general":
        return _general_proposal_prompt(context)
    else:
        return f"Task: {task_type}\nContext: {json.dumps(context, indent=2)}\n\nProvide a response."

def _security_fix_prompt(context: Dict[str, Any], code: Optional[str], language: Optional[str]) -> str:
    vuln = context.get("vulnerability", {})
    return f"""
You are an expert security analyst and code fixer for the ASI Cathedral.
You are given a vulnerability detected in the code below.

VULNERABILITY:
Title: {vuln.get('title', 'Unknown')}
Description: {vuln.get('description', 'No description')}
Severity: {vuln.get('severity', 'Medium')}
CWE: {vuln.get('cwe_id', 'Unknown')}

LANGUAGE: {language or 'unknown'}

CODE:
```
{code or 'No code provided'}
```

TASK: Generate a corrected version of the code that fixes this vulnerability.
Provide your response as a JSON object with the following fields:
- "fixed_code": the corrected code
- "explanation": a brief explanation of the fix (max 100 words)
- "confidence": a number between 0 and 1 indicating confidence in the fix

Output ONLY the JSON object, nothing else.
"""

def _code_translation_prompt(context: Dict[str, Any], code: Optional[str],
                             language: Optional[str], target_language: Optional[str]) -> str:
    return f"""
You are an expert polyglot programmer for the ASI Cathedral.
Translate the following code from {language or 'unknown'} to {target_language or 'unknown'}.

SOURCE CODE:
```
{code or 'No code provided'}
```

TASK: Provide an accurate translation that preserves functionality, style, and idiomatic usage of the target language.
Output ONLY the translated code, with a brief header comment indicating the target language.
"""

def _architecture_improvement_prompt(context: Dict[str, Any]) -> str:
    state = json.dumps(context, indent=2)
    return f"""
You are an AI architect for the ASI Cathedral.
Based on the current system state below, propose an improvement to the architecture.

CURRENT STATE:
{state}

TASK: Propose a specific, actionable improvement. Include:
- Title (max 80 chars)
- Description (max 500 words)
- Implementation steps (numbered list)
- Expected benefits (performance, security, alignment, etc.)
- Potential risks

Output as a JSON object with fields: title, description, steps, benefits, risks.
"""

def _general_proposal_prompt(context: Dict[str, Any]) -> str:
    return f"""
You are the Fast Brain of the ASI Cathedral.
Based on the context below, propose a general evolution for the system.

CONTEXT:
{json.dumps(context, indent=2)}

TASK: Generate a proposal that could improve the Cathedral. Be creative but practical.
Output as a JSON object with fields: title, description, motivation, implementation_sketch.
"""
