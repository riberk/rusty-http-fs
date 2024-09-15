use std::marker::PhantomData;

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{de::DeserializeOwned, Serialize};

use crate::config::app_config::TokenSecretsConfig;

use super::{access_token_claims::AccessTokenClaims, refresh_token_claims::RefreshTokenClaims};

pub struct JwtTokenEncoder<T> {
    alg: jsonwebtoken::Algorithm,
    key: EncodingKey,
    _d: PhantomData<T>,
}

pub struct JwtTokenDecoder<T> {
    alg: jsonwebtoken::Algorithm,
    key: DecodingKey,
    _d: PhantomData<T>,
}

const ALG: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;

pub struct TokensEncDec {
    pub access: EncDecPair<AccessTokenClaims>,
    pub refresh: EncDecPair<RefreshTokenClaims>,
}

impl TokensEncDec {
    pub fn from_config(config: &TokenSecretsConfig) -> Self {
        Self {
            access: EncDecPair::from_secret(config.access_secret()),
            refresh: EncDecPair::from_secret(config.refresh_secret()),
        }
    }
}

pub struct EncDecPair<T> {
    pub encoder: JwtTokenEncoder<T>,
    pub decoder: JwtTokenDecoder<T>,
}

impl<T> EncDecPair<T> {
    pub fn from_secret(secret: &str) -> Self {
        Self {
            encoder: JwtTokenEncoder {
                alg: ALG,
                key: EncodingKey::from_secret(secret.as_bytes()),
                _d: PhantomData,
            },
            decoder: JwtTokenDecoder {
                alg: ALG,
                key: DecodingKey::from_secret(secret.as_bytes()),
                _d: PhantomData,
            },
        }
    }
}

impl<T: Serialize> JwtTokenEncoder<T> {
    pub fn encode(&self, value: &T) -> Result<String, jsonwebtoken::errors::Error> {
        let header = jsonwebtoken::Header::new(self.alg);
        jsonwebtoken::encode(&header, value, &self.key)
    }
}

impl<T: DeserializeOwned> JwtTokenDecoder<T> {
    pub fn decode<S: AsRef<str>>(
        &self,
        token: S,
    ) -> Result<jsonwebtoken::TokenData<T>, jsonwebtoken::errors::Error> {
        let token = token.as_ref();
        let validation = jsonwebtoken::Validation::new(self.alg);
        jsonwebtoken::decode::<T>(token, &self.key, &validation)
    }
}
