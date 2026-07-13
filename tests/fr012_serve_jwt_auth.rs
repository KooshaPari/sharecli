//! FR: FR-012 — Serve HTTP Federated AuthN (JWT / JWKS resource server).
//!
//! AC-012.1 Valid RS256 JWT with matching iss/aud → authorized
//! AC-012.2 Expired JWT → rejected (`jwt_expired`)
//! AC-012.3 Wrong aud/iss rejected; probes stay public (unit coverage)

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Serialize;
use sharecli::config::{ServeConfig, ServeJwtConfig};
use sharecli::serve_auth::ServeAuth;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: i64,
    nbf: i64,
}

struct TestKeys {
    encoding: EncodingKey,
    jwks: String,
}

fn test_keys() -> &'static TestKeys {
    static KEYS: OnceLock<TestKeys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut rng = rand::rngs::OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).expect("rsa key");
        let public = RsaPublicKey::from(&private);
        let pem = private
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .expect("pem")
            .to_string();
        let n = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            public.n().to_bytes_be(),
        );
        let e = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            public.e().to_bytes_be(),
        );
        let jwks = format!(
            r#"{{"keys":[{{"kty":"RSA","use":"sig","alg":"RS256","kid":"test-key-1","n":"{n}","e":"{e}"}}]}}"#
        );
        TestKeys {
            encoding: EncodingKey::from_rsa_pem(pem.as_bytes()).expect("encoding key"),
            jwks,
        }
    })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn mint(keys: &TestKeys, iss: &str, aud: &str, exp_offset: i64) -> String {
    let claims = Claims {
        sub: "fr012-user".into(),
        iss: iss.into(),
        aud: aud.into(),
        exp: now() + exp_offset,
        nbf: now() - 10,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-1".into());
    encode(&header, &claims, &keys.encoding).expect("encode")
}

fn jwt_auth(keys: &TestKeys) -> ServeAuth {
    let cfg = ServeConfig {
        bearer_token: None,
        auth_mode: Some("jwt".into()),
        jwt: Some(ServeJwtConfig {
            issuer: "https://idp.example/".into(),
            audience: "sharecli-serve".into(),
            jwks_path: None,
            jwks: Some(keys.jwks.clone()),
        }),
    };
    unsafe {
        std::env::remove_var("SHARECLI_SERVE_TOKEN");
        std::env::remove_var("SHARECLI_SERVE_AUTH_MODE");
    }
    ServeAuth::from_env_or_config(&cfg).expect("jwt config")
}

#[test]
fn fr012_valid_jwt_authorized() {
    // FR: FR-012 AC-012.1
    let keys = test_keys();
    let auth = jwt_auth(keys);
    assert_eq!(auth.mode_label(), "jwt");
    let token = mint(keys, "https://idp.example/", "sharecli-serve", 3600);
    let sub = auth
        .check_authorization(Some(&format!("Bearer {token}")))
        .expect("valid jwt");
    assert_eq!(sub.as_deref(), Some("fr012-user"));
}

#[test]
fn fr012_expired_jwt_rejected() {
    // FR: FR-012 AC-012.2
    let keys = test_keys();
    let auth = jwt_auth(keys);
    let token = mint(keys, "https://idp.example/", "sharecli-serve", -120);
    let err = auth
        .check_authorization(Some(&format!("Bearer {token}")))
        .unwrap_err();
    assert_eq!(err, "jwt_expired");
}

#[test]
fn fr012_wrong_audience_and_issuer_rejected() {
    // FR: FR-012 AC-012.3
    let keys = test_keys();
    let auth = jwt_auth(keys);
    let bad_aud = mint(keys, "https://idp.example/", "other", 3600);
    assert_eq!(
        auth.check_authorization(Some(&format!("Bearer {bad_aud}")))
            .unwrap_err(),
        "jwt_invalid_aud"
    );
    let bad_iss = mint(keys, "https://evil.example/", "sharecli-serve", 3600);
    assert_eq!(
        auth.check_authorization(Some(&format!("Bearer {bad_iss}")))
            .unwrap_err(),
        "jwt_invalid_iss"
    );
}
