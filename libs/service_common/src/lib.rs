mod constants;
pub mod extract_session_headers;
pub use constants::{
    BODY_SIZE_LIMIT, CORS_ALLOW_ORIGIN, PUBLIC_TENANT, SERVICE_APPLICATION_NAME,
    SERVICE_COLLECTION_NAME, SERVICE_CONFIG_VOLUME, SERVICE_DATA_VOLUME, SERVICE_HOST,
    SERVICE_PORT,
};
use serde::Serialize;
use serde_json::{json, Value};

#[deprecated]
pub fn to_value<T: Serialize + core::fmt::Debug>(data: T) -> Value {
    match serde_json::to_value(&data) {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("error serializing {:?}, error: {e}", &data);
            json!({})
        }
    }
}
pub fn to_json_string<T: Serialize + core::fmt::Debug>(data: T) -> String {
    match serde_json::to_string(&data) {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("error serialing {:?}, error: {e}", &data);
            "{}".into()
        }
    }
}

pub struct IdGenerator;

impl IdGenerator {
    pub fn get(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
