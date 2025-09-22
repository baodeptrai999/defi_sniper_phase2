#[macro_export]
macro_rules!  solscan{
    ($($signature: expr)*) => {
        format!("https://solanabeach.io/transaction/{}", $signature)
    };
}