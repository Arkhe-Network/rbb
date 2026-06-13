import argparse
import sys
import json
import time
from typing import Dict, Any

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--method", type=str, default="swir")
    parser.add_argument("--prompt", type=str, required=True)
    args = parser.parse_args()

    print(f"Running prompt: {args.prompt} with method: {args.method}")
    print("Mocking successful completion")

    mock_data = {
        "run_id": "run_mock_001",
        "entropy_series": [0.1, 0.2, 0.3, 0.4, 0.1],
        "switch_points": [2, 4],
        "switch_count": 2,
        "output_tokens": 5,
        "latency_total_us": 15000
    }
    with open("swir_metrics.jsonl", "a") as f:
        f.write(json.dumps(mock_data) + "\n")

if __name__ == "__main__":
    main()
