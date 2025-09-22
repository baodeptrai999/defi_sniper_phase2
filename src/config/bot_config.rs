use std::sync::Arc;
use crate::*;

use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::{keypair::Keypair, Signer}};
use solana_commitment_config::CommitmentConfig;

use crate::CONFIG;

//Wallet key
pub static SIGNER_KEYPAIR: Lazy<Keypair> = Lazy::new(||{
  let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_credential.private_key);
  wallet
});

pub static SIGNER_PUBKEY: Lazy<Pubkey> = Lazy::new(||{
  let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_credential.private_key);
  wallet.pubkey()
});


//RPC endpoint
pub static RPC_ENDPOINT: Lazy<Arc<RpcClient>> = Lazy::new(||{
  Arc::new(RpcClient::new_with_commitment(CONFIG.connection_config.rpc_endpoint.clone(), CommitmentConfig { commitment: solana_commitment_config::CommitmentLevel::Processed }))
});
pub static GRPC_ENDPOINT: Lazy<String> = Lazy::new(||CONFIG.connection_config.grpc_endpoint.clone());
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(||CONFIG.connection_config.grpc_token.clone());


//Confirm service
pub static CONFIRM_SERVICE: Lazy<String> = Lazy::new(|| CONFIG.relayer.confirm_service.clone());
pub static JITO_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer.jito_api_key.clone());
pub static NOZOMI_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer.nozomi_api_key.clone());
pub static ZERO_SLOT_API_KEY: Lazy<String> = Lazy::new(||CONFIG.relayer.zero_slot_key.clone());


//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(||CONFIG.buy_setting.buy_amount_sol);

//Slippage
pub static SLIPPAGE: Lazy<u32> = Lazy::new(||CONFIG.slippage);

pub fn show_bot_settings(){
  log!("Public key: {:?}", SIGNER_PUBKEY);
  log!("Confirm service: {:?}", CONFIRM_SERVICE);
  log!("Buy settings: {:?}", CONFIG.buy_setting);
  log!("Slippage: {:?}%", SLIPPAGE);
  log!("Grpc endpoint: {:?}", GRPC_ENDPOINT);
  log!("Grpc token: {:?}", GRPC_TOKEN);
}

