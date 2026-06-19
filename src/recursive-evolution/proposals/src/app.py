import os
import structlog
from flask import Flask, request, jsonify
from flask_cors import CORS
from dotenv import load_dotenv
from src.orchestrator import InferenceOrchestrator
from src.validators import validate_generate_request, validate_fix_request
from src.governance_integration import GovernanceIntegration

load_dotenv()

structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.JSONRenderer()
    ]
)
logger = structlog.get_logger()

app = Flask(__name__)
CORS(app)

orchestrator = InferenceOrchestrator(
    venice_api_key=os.getenv("VENICE_API_KEY", "dummy"),
    bittensor_network=os.getenv("BITTENSOR_NETWORK", "finney"),
    use_local_fallback=os.getenv("USE_LOCAL_FALLBACK", "true").lower() == "true"
)

governance = GovernanceIntegration(
    orchestrator_url=os.getenv("ORCHESTRATOR_URL", "http://localhost:8080")
)

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "healthy", "version": "0.1.0"})

@app.route("/generate", methods=["POST"])
def generate_proposal():
    data = request.get_json()
    if not data:
        return jsonify({"error": "Missing JSON body"}), 400

    errors = validate_generate_request(data)
    if errors:
        return jsonify({"errors": errors}), 400

    logger.info("Generating proposal", task_type=data.get("task_type"))
    try:
        result = orchestrator.generate(
            task_type=data.get("task_type"),
            context=data.get("context", {}),
            language=data.get("language"),
            code=data.get("code"),
            target_language=data.get("target_language")
        )
        if result.get("proposal"):
            governance.submit_proposal(result["proposal"])
        return jsonify(result), 200
    except Exception as e:
        logger.error("Generation failed", error=str(e), exc_info=True)
        return jsonify({"error": str(e)}), 500

@app.route("/fix", methods=["POST"])
def generate_fix():
    data = request.get_json()
    errors = validate_fix_request(data)
    if errors:
        return jsonify({"errors": errors}), 400

    try:
        result = orchestrator.generate_fix(
            finding=data["finding"],
            code=data["code"],
            language=data.get("language", "unknown")
        )
        return jsonify(result), 200
    except Exception as e:
        logger.error("Fix generation failed", error=str(e), exc_info=True)
        return jsonify({"error": str(e)}), 500

@app.route("/translate", methods=["POST"])
def translate_code():
    data = request.get_json()
    if not data or "code" not in data or "target_language" not in data:
        return jsonify({"error": "Missing required fields"}), 400

    try:
        result = orchestrator.generate(
            task_type="code_translation",
            context={"code": data["code"]},
            language=data.get("source_language"),
            target_language=data["target_language"]
        )
        return jsonify(result), 200
    except Exception as e:
        logger.error("Translation failed", error=str(e), exc_info=True)
        return jsonify({"error": str(e)}), 500

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    app.run(host="0.0.0.0", port=port)
