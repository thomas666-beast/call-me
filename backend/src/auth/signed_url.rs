use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// How long a signed playback URL stays valid.
const PLAYBACK_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Derive a signing key from the JWT secret.
///
/// We use a distinct label ("playback-url-v1") so that a leaked playback
/// URL can't be used to forge a session token, and vice versa — even
/// though both are derived from the same underlying secret.
fn signing_key(jwt_secret: &str) -> Vec<u8> {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(b"playback-url-v1:");
    hasher.update(jwt_secret.as_bytes());
    hasher.finalize().to_vec()
}

/// Sign a playback URL for a recording.
/// Returns (expires_at_unix, signature_hex).
pub fn sign(jwt_secret: &str, recording_id: &str) -> (i64, String) {
    let expires_at = now_unix() + PLAYBACK_TTL.as_secs() as i64;
    let message = format!("{}:{}", recording_id, expires_at);

    let mut mac = HmacSha256::new_from_slice(&signing_key(jwt_secret))
        .expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());

    (expires_at, sig)
}

/// Verify a signed playback URL. Returns Ok(()) if the signature is valid
/// and the URL has not expired.
pub fn verify(
    jwt_secret: &str,
    recording_id: &str,
    expires_at: i64,
    signature_hex: &str,
) -> Result<()> {
    if now_unix() > expires_at {
        anyhow::bail!("signed URL expired");
    }

    let message = format!("{}:{}", recording_id, expires_at);
    let mut mac = HmacSha256::new_from_slice(&signing_key(jwt_secret))
        .context("HMAC init failed")?;
    mac.update(message.as_bytes());

    let expected = hex::decode(signature_hex).context("invalid signature encoding")?;
    mac.verify_slice(&expected).context("signature mismatch")?;
    Ok(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
