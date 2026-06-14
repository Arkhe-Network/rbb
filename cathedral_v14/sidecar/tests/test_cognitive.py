import asyncio
import pytest
from unittest.mock import AsyncMock, patch
from sidecar.cognitive_substrate import CognitiveSubstrate

@pytest.mark.asyncio
async def test_cognitive_tick_mocked_zmq():
    # Moca a conexão ZMQ
    with patch('zmq.asyncio.Context') as mock_ctx:
        mock_socket = AsyncMock()
        mock_ctx.return_value.socket.return_value = mock_socket

        # Moca as respostas do ZMQ
        async def mock_send_request(command, payload):
            if command == "RECALL":
                return {"data": [{"text": "memory 1"}, {"text": "memory 2"}]}
            elif command == "INTROSPECT":
                return {"data": {"health": "nominal", "godelian_check": True}}
            elif command == "UPDATE_ENERGY":
                return {"data": {"current_budget": 15.0}}
            return {}

        substrate = CognitiveSubstrate(embed_dim=64)
        substrate._send_request = mock_send_request

        result = await substrate.process_cognitive_tick(
            prompt="Test prompt",
            gguf_output_text="Test response",
            gguf_tokens=100,
            gguf_embedding=[0.1] * 64
        )

        assert result["episodic_memories_found"] == 2
        assert result["health"] == "nominal"
        assert result["godelian_check"] is True
        assert result["energy_state"] == 15.0
        assert result["meta_params"]["learning_rate"] == 1e-7
        assert "simulated_reward" in result["simulated_outcome"]

        await substrate.close()
