use crate::*;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;

pub static PUMPFUN_USER_VOLUME_ACCUMULATOR: Lazy<Pubkey> = Lazy::new(|| {
    let (pda, _bump) = Pubkey::find_program_address(
        &[USER_VOLUME_ACCUMULATOR_SEED, SIGNER_PUBKEY.as_ref()],
        &PUMPFUN_PROGRAM_ID,
    );
    pda
});
