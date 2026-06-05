import unittest
from unittest.mock import MagicMock, patch
import sys
import os

# Add src to python path to import bridge
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../src')))

# Mocking external Cathedral dependencies
sys.modules['cathedral'] = MagicMock()
sys.modules['cathedral.axiarquia'] = MagicMock()
sys.modules['cathedral.zk'] = MagicMock()
sys.modules['cathedral.temporal_chain'] = MagicMock()

from bridge import FordefiBridge

class TestFordefiBridge(unittest.TestCase):
    @patch('bridge.requests.post')
    @patch('bridge.anchor')
    def test_submit_transaction_success(self, mock_anchor, mock_post):
        # Setup mocks
        mock_gate = MagicMock()
        mock_gate.validate_fordefi_transaction.return_value = True
        mock_gate.version = "1.0.0"

        mock_response = MagicMock()
        mock_response.json.return_value = {"id": "test_tx_id", "hash": "0x123"}
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        bridge = FordefiBridge(api_key="test_api_key", vault_id="test_vault", gate=mock_gate)

        # Override zk mock inside bridge since it was created in __init__
        bridge.zk.generate.return_value = MagicMock(merkle_root="0xabc", hash="hash123456")

        # Execute
        tx_id = bridge.submit_transaction(
            to="0xcontract",
            method="mint",
            args={"amount": 100}
        )

        # Assertions
        self.assertEqual(tx_id, "test_tx_id")
        mock_gate.validate_fordefi_transaction.assert_called_once()
        mock_post.assert_called_once()
        self.assertEqual(mock_anchor.call_count, 2) # Once for ZK, once for TemporalChain

if __name__ == '__main__':
    unittest.main()
