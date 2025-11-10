use crate::secrets::models::{Secret, SecretBytes, SecretDict};
use crate::secrets::utils::{create_secrets_dir, write_json, write_json_pretty};
use serde_json::Value;
use std::collections::BTreeMap;
use tracing::{debug, info, instrument, warn};

#[instrument(skip_all)]
pub fn summarise(caps: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let mut real: BTreeMap<i32, String> = BTreeMap::new();

    info!("Processing {} captured items...", caps.len());

    for (idx, cap) in caps.iter().enumerate() {
        let secret = if let Some(Value::String(s)) = cap.get("secret") {
            s.clone()
        } else {
            debug!("Item {} has no valid secret string", idx);
            continue;
        };

        let version = match cap.get("version") {
            Some(Value::Number(n)) => i32::try_from(n.as_i64().unwrap_or(0)).unwrap_or(0),
            Some(Value::String(s)) => s.parse::<i32>().unwrap_or(0),
            _ => {
                // Try to get version from obj
                cap.get("obj").and_then(|v| v.as_object()).map_or(0, |obj| {
                    match obj.get("version") {
                        Some(Value::Number(n)) => {
                            i32::try_from(n.as_i64().unwrap_or(0)).unwrap_or(0)
                        }
                        Some(Value::String(s)) => s.parse::<i32>().unwrap_or(0),
                        _ => 0,
                    }
                })
            }
        };

        if version > 0 {
            debug!("Found secret version {} at index {}", version, idx);
            real.insert(version, secret);
        } else {
            debug!("Skipping item {} - invalid version", idx);
        }
    }

    if real.is_empty() {
        warn!("No real secrets with valid version found");
        return Ok(());
    }

    info!("Found {} unique secret versions", real.len());

    let formatted_data: Vec<Secret> = real
        .iter()
        .map(|(version, secret)| Secret {
            version: *version,
            secret: secret.clone(),
        })
        .collect();

    let secret_bytes: Vec<SecretBytes> = real
        .iter()
        .map(|(version, secret)| SecretBytes {
            version: *version,
            secret: secret.chars().map(|c| c as i32).collect(),
        })
        .collect();

    let mut secret_dict: SecretDict = BTreeMap::new();
    for (version, secret) in &real {
        secret_dict.insert(
            version.to_string(),
            secret.chars().map(|c| c as i32).collect(),
        );
    }

    print_summary(&formatted_data, &secret_bytes, &secret_dict)?;
    write_files(&formatted_data, &secret_bytes, &secret_dict)?;

    Ok(())
}

fn print_summary(
    formatted_data: &[Secret],
    secret_bytes: &[SecretBytes],
    secret_dict: &SecretDict,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== List of extracted secrets ===");

    for secret in formatted_data {
        info!("v{}: (length: {})", secret.version, secret.secret.len());
    }

    debug!(
        "Plain secrets (JSON):\n{}",
        serde_json::to_string_pretty(formatted_data)?
    );
    debug!(
        "Secret bytes (JSON):\n{}",
        serde_json::to_string_pretty(secret_bytes)?
    );
    debug!(
        "Secret dict (JSON):\n{}",
        serde_json::to_string_pretty(secret_dict)?
    );

    Ok(())
}

fn write_files(
    formatted_data: &[Secret],
    secret_bytes: &[SecretBytes],
    secret_dict: &SecretDict,
) -> Result<(), Box<dyn std::error::Error>> {
    create_secrets_dir()?;

    write_json_pretty("secrets/secrets.json", &formatted_data)?;
    info!("Wrote plain secrets to secrets/secrets.json");

    write_json("secrets/secretBytes.json", &secret_bytes)?;
    info!("Wrote secret bytes array to secrets/secretBytes.json");

    write_json("secrets/secretDict.json", &secret_dict)?;
    info!("Wrote secret bytes dict to secrets/secretDict.json");

    Ok(())
}
