import bittensor as bt
import structlog

logger = structlog.get_logger()

class BittensorClient:
    def __init__(self, network: str = "finney"):
        self.network = network
        self.subtensor = bt.Subtensor(network=network)
        self.wallet = bt.Wallet()
        self.metagraph = None

    def _get_metagraph(self, netuid: int):
        if self.metagraph is None or self.metagraph.netuid != netuid:
            self.metagraph = self.subtensor.metagraph(netuid)
        return self.metagraph

    def infer(self, prompt: str, subnet: str = "SN96") -> str:
        netuid = 96 if subnet == "SN96" else 92
        metagraph = self._get_metagraph(netuid)
        miners = [uid for uid, neuron in enumerate(metagraph.neurons) if neuron.active]
        if not miners:
            raise RuntimeError("No active miners in subnet")
        target_uid = miners[0]
        axon = metagraph.neurons[target_uid].axon_info
        logger.info("Bittensor inference", subnet=subnet, target_uid=target_uid)
        # Simulação: em produção usar bt.dendrite
        return f"[Bittensor {subnet}] Simulated response for prompt: {prompt[:100]}..."
