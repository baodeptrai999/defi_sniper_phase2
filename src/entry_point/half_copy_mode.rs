use pumpfun_sniper::*;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use spl_associated_token_account::get_associated_token_address;
pub fn main() {
    let user = pubkey!("CzsiD9jrDihBAgGt6EFkYWmx4oFw8dqoTAmyrtS3hcRz");
    let mint = pubkey!("GXNa8j8fyVSqsi59UDzrmNoPKh49GxL9d1LZBTDapump");
    let (user_volume_accumulator, _) =
        Pubkey::find_program_address(&[USER_VOLUME_ACCUMULATOR_SEED, user.as_ref()], &PUMPFUN_PROGRAM_ID);

        // let x = get_associated_token_address(&creator_vault, &mint);
        println!("{}", user_volume_accumulator);
}
