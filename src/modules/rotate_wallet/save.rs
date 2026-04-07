use crate::*;
use std::{fs, path::Path};

/// Save rotation wallet details as a JSON file under src/assets/rotation/.
/// All fields are optional so partial results can be saved on failure.
pub fn save_rotation_json(
    status: &str,
    error_msg: Option<&str>,
    old_sol_pubkey: &str,
    old_sol_private_key: &str,
    bnb_address: Option<&str>,
    bnb_private_key: Option<&str>,
    new_sol_pubkey: Option<&str>,
    new_sol_private_key: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut json = serde_json::json!({
        "status": status,
        "previous_sol_wallet": {
            "pubkey": old_sol_pubkey,
            "private_key": old_sol_private_key
        }
    });

    if let Some(e) = error_msg {
        json["error"] = serde_json::json!(e);
    }

    if let (Some(addr), Some(pk)) = (bnb_address, bnb_private_key) {
        json["bnb_wallet"] = serde_json::json!({
            "address": addr,
            "private_key": format!("0x{}", pk)
        });
    }

    if let (Some(pub_k), Some(priv_k)) = (new_sol_pubkey, new_sol_private_key) {
        json["rotated_sol_wallet"] = serde_json::json!({
            "pubkey": pub_k,
            "private_key": priv_k
        });
    }

    let dir = Path::new("src/assets/rotation");
    fs::create_dir_all(dir)?;

    let now = chrono::Local::now();
    let filename = format!(
        "wallet_rotation_{}.json",
        now.format("%Y_%m_%d_%H_%M_%S")
    );
    let path = dir.join(&filename);
    fs::write(&path, serde_json::to_string_pretty(&json)?)?;

    let path_str = path.to_string_lossy().to_string();
    info!("[ROTATE] Rotation JSON saved: {}", path_str);
    Ok(path_str)
}
