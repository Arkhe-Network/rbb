// src/integrations/picnic.rs
//! Integração com Picnic (DeFi Basket) para gestão de royalties com yield.

use ethers::prelude::*;
use tracing::info;

abigen!(
    IPicnicBasket,
    r#"[
        function deposit(uint256 amount, address receiver) external returns (uint256 shares)
        function totalAssets() external view returns (uint256)
        function distributeRewards(address[] calldata recipients, uint256[] calldata amounts) external
        function owner() external view returns (address)
    ]"#
);

#[derive(Debug, Clone)]
pub struct PicnicRoyaltyManager {
    client: SignerMiddleware<Provider<Http>, Wallet<ethers::core::k256::ecdsa::SigningKey>>,
    basket_address: Address,
}

impl PicnicRoyaltyManager {
    pub fn new(
        rpc_url: &str,
        private_key: &str,
        basket_address: Address,
    ) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| format!("Erro ao conectar provider: {}", e))?;

        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| format!("Chave privada inválida: {}", e))?;

        let client = SignerMiddleware::new(provider, wallet);

        Ok(Self {
            client,
            basket_address,
        })
    }

    /// Verifica se o basket existe e está ativo
    pub async fn verify_basket(&self) -> Result<(), String> {
        let contract = IPicnicBasket::new(self.basket_address, self.client.clone().into());
        let _owner = contract
            .owner()
            .call()
            .await
            .map_err(|e| format!("Basket inválido ou inexistente: {}", e))?;
        Ok(())
    }

    /// Converte um endereço Ethereum (string) para Address.
    pub fn parse_address(addr: &str) -> Result<Address, String> {
        addr.parse().map_err(|_| format!("Endereço inválido: {}", addr))
    }
}
