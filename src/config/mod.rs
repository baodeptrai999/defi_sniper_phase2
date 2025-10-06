use std::fs;
use once_cell::sync::Lazy;
use serde::Deserialize;

pub mod toml_config;
pub mod bot_config;
pub mod trade_setting;

pub use toml_config::*;
pub use bot_config::*;
pub use trade_setting::*;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub mode: ModeConfig,
  pub wallet_config: WalletCredentialConfig,
  pub target_config: TargetConfig,
  pub connection_config: ConnectionConfig,
  pub relayer_config: RelayerConfig,
  pub buy_setting: BuySetting,
  pub sell_setting: SellSetting,
  pub slippage_config: SlippageConfig,
  pub fee_config: FeeConfig,
  pub filter_setting: FilterSetting,
  pub monitor_setting: MonitorConfig,
  pub shut_down_setting: ShutDownConfig
}

pub static CONFIG: Lazy<Config> = Lazy::new(||{
  let content = fs::read_to_string("Config.toml").expect("Failed to read Config.toml file");
  toml::from_str(&content).expect("Failed to parse config file.")
});