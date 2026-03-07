use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    alg: String,
    typ: String,
}

fn jwt_secret() -> Result<String, String> {
    dotenvy::dotenv().ok();
    std::env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET environment variable is required".to_string())
}

fn sign(input: &str, secret: &str) -> Result<String, String> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| format!("invalid key: {e}"))?;
    mac.update(input.as_bytes());
    let signature = mac.finalize().into_bytes();
    Ok(URL_SAFE_NO_PAD.encode(signature))
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.as_bytes()
        .iter()
        .zip(right.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

pub fn create_jwt(user_id: Uuid, expiration_seconds: usize) -> String {
    let secret = jwt_secret().expect("JWT_SECRET environment variable is required");

    let header = Header {
        alg: "HS256".to_string(),
        typ: "JWT".to_string(),
    };
    let claims = Claims {
        sub: user_id,
        exp: expiration_seconds,
    };

    let header_json = serde_json::to_vec(&header).expect("failed to serialize JWT header");
    let claims_json = serde_json::to_vec(&claims).expect("failed to serialize JWT claims");

    let header_b64 = URL_SAFE_NO_PAD.encode(header_json);
    let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json);
    let signing_input = format!("{header_b64}.{claims_b64}");
    let signature = sign(&signing_input, &secret).expect("failed to sign JWT");

    format!("{signing_input}.{signature}")
}

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let secret = jwt_secret()?;

    let mut parts = token.split('.');
    let header_b64 = parts
        .next()
        .ok_or_else(|| "invalid token: missing header".to_string())?;
    let claims_b64 = parts
        .next()
        .ok_or_else(|| "invalid token: missing claims".to_string())?;
    let signature = parts
        .next()
        .ok_or_else(|| "invalid token: missing signature".to_string())?;

    if parts.next().is_some() {
        return Err("invalid token: too many segments".to_string());
    }

    let signing_input = format!("{header_b64}.{claims_b64}");
    let expected_signature = sign(&signing_input, &secret)?;

    if !constant_time_eq(signature, &expected_signature) {
        return Err("invalid token signature".to_string());
    }

    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_b64)
        .map_err(|e| format!("invalid header encoding: {e}"))?;
    let header: Header =
        serde_json::from_slice(&header_bytes).map_err(|e| format!("invalid header json: {e}"))?;

    if header.alg != "HS256" || header.typ != "JWT" {
        return Err("unsupported token header".to_string());
    }

    let claims_bytes = URL_SAFE_NO_PAD
        .decode(claims_b64)
        .map_err(|e| format!("invalid claims encoding: {e}"))?;
    let claims: Claims =
        serde_json::from_slice(&claims_bytes).map_err(|e| format!("invalid claims json: {e}"))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system time error: {e}"))?
        .as_secs() as usize;

    if claims.exp <= now {
        return Err("token expired".to_string());
    }

    Ok(claims)
}
