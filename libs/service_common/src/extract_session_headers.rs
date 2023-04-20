use std::ops::Deref;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use mu_rust_common::{SessionQueryHeaders, HEADER_MU_CALL_ID, HEADER_MU_SESSION_ID};

pub struct ExtractSession(pub SessionQueryHeaders);

impl Deref for ExtractSession {
    type Target = SessionQueryHeaders;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[axum::async_trait]
impl<B> FromRequestParts<B> for ExtractSession
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let session_header = SessionQueryHeaders {
            call_id: req
                .headers
                .get(HEADER_MU_CALL_ID)
                .and_then(|c| c.to_str().ok())
                .filter(|u| !u.trim().is_empty())
                .map(|c| c.to_string()),
            session_id: req
                .headers
                .get(HEADER_MU_SESSION_ID)
                .and_then(|c| c.to_str().ok())
                .filter(|u| !u.trim().is_empty())
                .map(|c| c.to_string()),
        };
        Ok(ExtractSession(session_header))
    }
}
