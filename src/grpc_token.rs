//! gRPC authentication token management using [JWT](https://en.wikipedia.org/wiki/JSON_Web_Token).
//!
//! Provides [`GrpcToken`] for creating and verifying JWT-based authentication tokens
//! for gRPC services. Each [`GrpcToken`] instance generates its own HMAC-SHA256 key,
//! so tokens can only be verified by the same instance that created them.
//!
//! # Example
//!
//! ```
//! use databend_base::grpc_token::{GrpcClaim, GrpcToken};
//!
//! let grpc_token = GrpcToken::create();
//!
//! let claim = GrpcClaim { username: "alice".to_string() };
//! let token = grpc_token.try_create_token(claim).unwrap();
//!
//! let verified = grpc_token.try_verify_token(&token).unwrap();
//! assert_eq!(verified.username, "alice");
//! ```

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use rand::RngCore;

const TOKEN_TTL_SECS: u64 = 3650 * 24 * 60 * 60;

/// Claims embedded in the JWT token payload.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GrpcClaim {
    /// The authenticated user's identifier.
    pub username: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct WireClaims {
    username: String,
    exp: u64,
}

/// JWT token manager for gRPC authentication.
///
/// Cloning shares the same key, allowing multiple references to create and
/// verify tokens interchangeably.
#[derive(Clone)]
pub struct GrpcToken {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl GrpcToken {
    /// Creates a new token manager with a randomly generated HMAC-SHA256 key.
    pub fn create() -> Self {
        let mut secret = [0u8; 32];
        rand::rng().fill_bytes(&mut secret);
        Self {
            encoding_key: EncodingKey::from_secret(&secret),
            decoding_key: DecodingKey::from_secret(&secret),
        }
    }

    /// Creates a signed JWT token valid for 10 years.
    pub fn try_create_token(
        &self,
        claim: GrpcClaim,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX epoch")
            .as_secs()
            + TOKEN_TTL_SECS;
        let wire = WireClaims {
            username: claim.username,
            exp,
        };
        encode(&Header::new(Algorithm::HS256), &wire, &self.encoding_key)
    }

    /// Verifies a token signature and expiration, returning the embedded claim.
    pub fn try_verify_token(&self, token: &str) -> Result<GrpcClaim, jsonwebtoken::errors::Error> {
        let data = decode::<WireClaims>(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(GrpcClaim {
            username: data.claims.username,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(name: &str) -> GrpcClaim {
        GrpcClaim {
            username: name.to_string(),
        }
    }

    #[test]
    fn test_create_and_verify() {
        let t = GrpcToken::create();
        let token = t.try_create_token(claim("alice")).unwrap();

        assert_eq!(t.try_verify_token(&token).unwrap().username, "alice");
    }

    #[test]
    fn test_cloned_manager_shares_key() {
        let t1 = GrpcToken::create();
        let t2 = t1.clone();

        let token = t1.try_create_token(claim("bob")).unwrap();
        assert_eq!(t2.try_verify_token(&token).unwrap().username, "bob");
    }

    #[test]
    fn test_different_managers_reject() {
        let t1 = GrpcToken::create();
        let t2 = GrpcToken::create();

        let token = t1.try_create_token(claim("alice")).unwrap();
        assert!(t2.try_verify_token(&token).is_err());
    }

    #[test]
    fn test_invalid_token() {
        let t = GrpcToken::create();
        assert!(t.try_verify_token("invalid").is_err());
        assert!(t.try_verify_token("").is_err());
    }
}
