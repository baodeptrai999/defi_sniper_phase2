use std::sync::Arc;
use crate::*;

use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::{keypair::Keypair, Signer}, commitment_config::CommitmentConfig, commitment_config::CommitmentLevel};

use crate::CONFIG;

//Wallet key
pub static SIGNER_KEYPAIR: Lazy<Keypair> = Lazy::new(||{
  let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
  wallet
});

pub static SIGNER_PUBKEY: Lazy<Pubkey> = Lazy::new(||{
  let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
  wallet.pubkey()
});


//RPC endpoint
pub static RPC_ENDPOINT: Lazy<String> = Lazy::new(||CONFIG.connection_config.rpc_endpoint.clone());
pub static RPC_CLIENT: Lazy<Arc<RpcClient>> = Lazy::new(||{
  Arc::new(RpcClient::new_with_commitment(CONFIG.connection_config.rpc_endpoint.clone(), CommitmentConfig { commitment: CommitmentLevel::Processed }))
});
pub static GRPC_ENDPOINT: Lazy<String> = Lazy::new(||CONFIG.connection_config.grpc_endpoint.clone());
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(||CONFIG.connection_config.grpc_token.clone());


//Confirm service
pub static CONFIRM_SERVICE: Lazy<String> = Lazy::new(|| CONFIG.relayer_config.confirm_service.clone());
pub static JITO_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer_config.jito_api_key.clone());
pub static NOZOMI_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer_config.nozomi_api_key.clone());
pub static ZERO_SLOT_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer_config.zero_slot_key.clone());


//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(||CONFIG.buy_setting.buy_amount_sol);

//Slippage
pub static SLIPPAGE: Lazy<u32> = Lazy::new(||CONFIG.slippage_config.slippage);

//Fee config
pub static PRIORITY_FEE: Lazy<(u64, u64, f64)> = Lazy::new(|| {
    let cu: u64 = CONFIG.fee_config.cu;
    let priority_fee_micro_lamport = CONFIG.fee_config.priority_fee_micro_lamport;

    let third_party_fee = CONFIG.fee_config.third_party_fee;

    (cu, priority_fee_micro_lamport, third_party_fee)
});

pub fn show_bot_settings(){
  log!("Public key: {:?}", *SIGNER_PUBKEY);
  log!("Confirm service: {:?}", *CONFIRM_SERVICE);
  log!("Buy settings: {:?}", CONFIG.buy_setting);
  log!("Slippage: {:?}%", *SLIPPAGE);
  log!("Grpc endpoint: {:?}", *GRPC_ENDPOINT);
  log!("Grpc token: {:?}", *GRPC_TOKEN);
  log!("RPC endpoint: {:?}", *RPC_ENDPOINT);
}

