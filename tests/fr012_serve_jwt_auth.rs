//! FR-012 — Serve HTTP Federated AuthN (JWT / JWKS resource server).
//!
//! AC-012.1 Valid RS256 JWT with matching iss/aud → authorized
//! AC-012.2 Expired JWT → rejected (`jwt_expired`)
//! AC-012.3 Probe paths remain public (unit coverage in serve_auth); wrong aud/iss rejected

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use sharecli::config::{ServeConfig, ServeJwtConfig};
use sharecli::serve_auth::ServeAuth;
use std::time::{SystemTime, UNIX_EPOCH};

const RSA_PRIVATE: &str = include_str!("fixtures/jwt_test_rsa_private.pem");
const JWKS: &str = include_str!("fixtures/jwt_test_jwks.json");

#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: i64,
    nbf: i64,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn mint(iss: &str, aud: &str, exp_offset: i64) -> String {
    let claims = Claims {
        sub: "fr012-user".into(),
        iss: iss.into(),
        aud: aud.into(),
        exp: now() + exp_offset,
        nbf: now() - 10,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-1".into());
    encode(&header, &claims, &EncodingKey::from_rsa_pem(RSA_PRIVATE.as_bytes()).expect("pem"))
        .expect("encode")
}

fn jwt_auth() -> ServeAuth {
    // Construct via public config path with explicit mode (no SHARECLI_SERVE_TOKEN).
    let cfg = ServeConfig {
        bearer_token: None,
        auth_mode: Some("jwt".into()),
        jwt: Some(ServeJwtConfig {
            issuer: "https://idp.example/".into(),
            audience: "sharecli-serve".into(),
            jwks_path: None,
            jwks: Some(JWKS.to_string()),
        }),
    };
    // Clear env so acceptance is deterministic even if other tests set it.
    unsafe {
        std::env::remove_var("SHARECLI_SERVE_TOKEN");
        std::env::remove_var("SHARECLI_SERVE_AUTH_MODE");
    }
    ServeAuth::from_env_or_config(&cfg).expect("jwt config")
}

#[test]
fn fr012_valid_jwt_authorized() {
    // FR-012 AC-012.1
    let auth = jwt_auth();
    assert_eq!(auth.mode_label(), "jwt");
    let token = mint("https://idp.example/", "sharecli-serve", 3600);
    let sub = auth.check_authorization(Some(&format!("Bearer {token}"))).expect("valid jwt");
    assert_eq!(sub.as_deref(), Some("fr012-user"));
}

#[test]
fn fr012_expired_jwt_rejected() {
    // FR-012 AC-012.2
    let auth = jwt_auth();
    let token = mint("https://idp.example/", "sharecli-serve", -120);
    let err = auth.check_authorization(Some(&format!("Bearer {token}"))).unwrap_err();
    assert_eq!(err, "jwt_expired");
}

#[test]
fn fr012_wrong_audience_and_issuer_rejected() {
    // FR-012 AC-012.3 (claim validation; probes covered in unit tests)
    let auth = jwt_auth();
    let bad_aud = mint("https://idp.example/", "other", 3600);
    assert_eq!(
        auth.check_authorization(Some(&format!("Bearer {bad_aud}"))).unwrap_err(),
        "jwt_invalid_aud"
    );
    let bad_iss = mint("https://evil.example/", "sharecli-serve", 3600);
    assert_eq!(
        auth.check_authorization(Some(&format!("Bearer {bad_iss}"))).unwrap_err(),
        "jwt_invalid_iss"
    );
}
