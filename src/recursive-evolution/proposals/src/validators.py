import json
import re
from typing import Dict, Any, Optional, List

def validate_and_parse_output(text: str, task_type: str) -> Optional[Dict[str, Any]]:
    json_match = re.search(r'\{.*\}', text, re.DOTALL)
    if not json_match:
        return None
    try:
        data = json.loads(json_match.group())
    except json.JSONDecodeError:
        return None

    if task_type == "security_fix":
        if "fixed_code" not in data or "explanation" not in data:
            return None
        return data
    elif task_type == "code_translation":
        if "translated_code" not in data:
            data["translated_code"] = text.strip()
        return data
    elif task_type in ["architecture_improvement", "general"]:
        required = ["title", "description"]
        if all(k in data for k in required):
            return data
        return None
    else:
        return data

def validate_generate_request(data: Dict[str, Any]) -> List[str]:
    errors = []
    if "task_type" not in data:
        errors.append("Missing 'task_type'")
    if data.get("task_type") not in ["security_fix", "architecture_improvement", "code_translation", "general"]:
        errors.append("Invalid 'task_type'")
    return errors

def validate_fix_request(data: Dict[str, Any]) -> List[str]:
    errors = []
    if "finding" not in data:
        errors.append("Missing 'finding'")
    if "code" not in data:
        errors.append("Missing 'code'")
    return errors
