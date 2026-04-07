/// Identify a System Program instruction by its u32 LE discriminator.
pub fn identify_system_program_ix(data: &[u8]) -> String {
    let disc = data
        .get(..4)
        .and_then(|b| <[u8; 4]>::try_from(b).ok())
        .map(u32::from_le_bytes);
    match disc {
        Some(0) => "System:CreateAccount".to_string(),
        Some(1) => "System:Assign".to_string(),
        Some(2) => "System:Transfer".to_string(),
        Some(3) => "System:CreateAccountWithSeed".to_string(),
        Some(4) => "System:AdvanceNonceAccount".to_string(),
        Some(5) => "System:WithdrawNonceAccount".to_string(),
        Some(6) => "System:InitializeNonceAccount".to_string(),
        Some(7) => "System:AuthorizeNonceAccount".to_string(),
        Some(8) => "System:Allocate".to_string(),
        Some(9) => "System:AllocateWithSeed".to_string(),
        Some(10) => "System:AssignWithSeed".to_string(),
        Some(11) => "System:TransferWithSeed".to_string(),
        Some(12) => "System:UpgradeNonceAccount".to_string(),
        _ => "System:Unknown".to_string(),
    }
}

/// Identify an Associated Token Program instruction by its single-byte discriminator.
pub fn identify_ata_program_ix(data: &[u8]) -> String {
    match data.first() {
        Some(&0) | None => "ATA:Create".to_string(),
        Some(&1) => "ATA:CreateIdempotent".to_string(),
        Some(&2) => "ATA:RecoverNested".to_string(),
        _ => "ATA:Unknown".to_string(),
    }
}

/// Identify a Token-2022 Program instruction by its single-byte discriminator.
pub fn identify_token2022_program_ix(data: &[u8]) -> String {
    match data.first() {
        Some(&0) => "Token2022:InitializeMint".to_string(),
        Some(&1) => "Token2022:InitializeAccount".to_string(),
        Some(&2) => "Token2022:InitializeMultisig".to_string(),
        Some(&3) => "Token2022:Transfer".to_string(),
        Some(&4) => "Token2022:Approve".to_string(),
        Some(&5) => "Token2022:Revoke".to_string(),
        Some(&6) => "Token2022:SetAuthority".to_string(),
        Some(&7) => "Token2022:MintTo".to_string(),
        Some(&8) => "Token2022:Burn".to_string(),
        Some(&9) => "Token2022:CloseAccount".to_string(),
        Some(&10) => "Token2022:FreezeAccount".to_string(),
        Some(&11) => "Token2022:ThawAccount".to_string(),
        Some(&12) => "Token2022:TransferChecked".to_string(),
        Some(&13) => "Token2022:ApproveChecked".to_string(),
        Some(&14) => "Token2022:MintToChecked".to_string(),
        Some(&15) => "Token2022:BurnChecked".to_string(),
        Some(&16) => "Token2022:InitializeAccount2".to_string(),
        Some(&17) => "Token2022:SyncNative".to_string(),
        Some(&18) => "Token2022:InitializeAccount3".to_string(),
        Some(&20) => "Token2022:InitializeMint2".to_string(),
        Some(&25) => "Token2022:InitializeImmutableOwner".to_string(),
        _ => "Token2022:Unknown".to_string(),
    }
}
