// src/bin/deploy_drex_contract.rs
use ethers::prelude::*;
use ethers::contract::ContractFactory;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Conecta à rede DREX (ou testnet)
    let provider = Provider::<Http>::try_from("https://drex-testnet.bcb.gov.br").map_err(|e| e.to_string())?;
    let wallet: LocalWallet = std::env::var("DREX_PRIVATE_KEY").unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000001".to_string()).parse().map_err(|e: <LocalWallet as std::str::FromStr>::Err| e.to_string())?;
    let client = SignerMiddleware::new(provider, wallet);

    // Carrega o bytecode e ABI do contrato
    // In a real environment, we'd use the compiled output of solc
    let bytecode = vec![];
    let abi = ethers::abi::Abi::default();

    let factory = ContractFactory::new(abi, bytecode.into(), client.into());

    // Parâmetros do contrato
    let drex_token: Address = std::env::var("DREX_TOKEN_ADDRESS").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()).parse().map_err(|_| "invalid address".to_string())?;
    let recipients: Vec<Address> = vec![
        "0x0000000000000000000000000000000000000001".parse().unwrap(),
        "0x0000000000000000000000000000000000000002".parse().unwrap(),
    ];
    let shares: Vec<U256> = vec![U256::from(7000), U256::from(3000)]; // 70% e 30%

    // Deploy
    let _contract = factory.deploy((drex_token, recipients, shares)).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    // println!("✅ Contrato RoyaltySplitter deployado em: {:?}", contract.address());

    Ok(())
}
