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

use jwt_simple::prelude::*;

/// Claims embedded in the JWT token payload.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GrpcClaim {
    /// The authenticated user's identifier.
    pub username: String,
}

/// JWT token manager for gRPC authentication.
///
/// Cloning shares the same key, allowing multiple references to create and
/// verify tokens interchangeably.
#[derive(Clone)]
pub struct GrpcToken {
    key: HS256Key,
}

impl GrpcToken {
    /// Creates a new token manager with a randomly generated HMAC-SHA256 key.
    pub fn create() -> Self {
        Self {
            key: HS256Key::generate(),
        }
    }

    /// Creates a signed JWT token valid for 10 years.
    pub fn try_create_token(&self, claim: GrpcClaim) -> Result<String, jwt_simple::Error> {
        self.key.authenticate(Claims::with_custom_claims(claim, Duration::from_days(3650)))
    }

    /// Verifies a token signature and expiration, returning the embedded claim.
    pub fn try_verify_token(&self, token: &str) -> Result<GrpcClaim, jwt_simple::Error> {
        Ok(self.key.verify_token::<GrpcClaim>(token, None)?.custom)
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
