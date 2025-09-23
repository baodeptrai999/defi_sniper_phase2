use solana_relayer_adapter_rust::{Jito, Nozomi, ZeroSlot};
use tokio::sync::OnceCell;

use crate::*;

pub static NOZOMI_CLIENT: OnceCell<Nozomi> = OnceCell::const_new();
pub static JITO_CLIENT: OnceCell<Jito> = OnceCell::const_new();
pub static ZERO_SLOT_CLIENT: OnceCell<ZeroSlot> = OnceCell::const_new();


pub async fn init_nozomi(){
  let nozomi = Nozomi::new_auto(NOZOMI_API_KEY.to_string()).await;
  NOZOMI_CLIENT.set(nozomi).unwrap();
}

pub async fn init_jito(){
  let jito = Jito::new_auto(Some(JITO_API_KEY.to_string())).await;
  JITO_CLIENT.set(jito).unwrap();
}

pub async fn init_zero_slot(){
  let zero_slot = ZeroSlot::new_auto(ZERO_SLOT_API_KEY.to_string()).await;
  ZERO_SLOT_CLIENT.set(zero_slot).unwrap();
}

