use std::fs;
use once_cell::sync::Lazy;
use serde::Deserialize;

pub mod toml_config;
pub mod bot_config;

pub use toml_config::*;
pub use bot_config::*;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub wallet_config: WalletCredentialConfig,
  pub connection_config: ConnectionConfig,
  pub relayer_config: RelayerConfig,
  pub buy_setting: BuySetting,
  pub slippage_config: SlippageConfig,
  pub fee_config: FeeConfig
}

pub static CONFIG: Lazy<Config> = Lazy::new(||{
  let content = fs::read_to_string("Config.toml").expect("Failed to read Config.toml file");
  toml::from_str(&content).expect("Failed to parse config file.")
});