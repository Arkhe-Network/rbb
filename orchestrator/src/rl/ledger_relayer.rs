use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

pub struct EthersRelayer {
    pub provider: Arc<Provider<Http>>,
    pub wallet: LocalWallet,
    pub contract_address: Address,
    pub transaction_queue: Arc<Mutex<VecDeque<TransactionRequest>>>,
}

impl EthersRelayer {
    pub async fn new(rpc_url: &str, private_key: &str, contract_address: Address) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| e.to_string())?;

        let wallet: LocalWallet = private_key.parse::<LocalWallet>()
            .map_err(|e| e.to_string())?
            .with_chain_id(provider.get_chainid().await.map_err(|e| e.to_string())?.as_u64());

        Ok(Self {
            provider: Arc::new(provider),
            wallet,
            contract_address,
            transaction_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub async fn enqueue_transaction(&self, tx: TransactionRequest) {
        let mut queue = self.transaction_queue.lock().await;
        queue.push_back(tx);
    }

    pub async fn process_queue(&self) -> Result<(), String> {
        let mut queue = self.transaction_queue.lock().await;
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());

        while let Some(tx) = queue.pop_front() {
            let pending_tx = client.send_transaction(tx, None).await.map_err(|e| e.to_string())?;
            let _receipt = pending_tx.await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
