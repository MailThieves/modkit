use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::store::StoreError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum DeviceType {
    Camera,
    Light,
    ContactSensor,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for DeviceType {
    fn from_row(row: &'r SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let dev_type = match row.try_get("device")? {
            "ContactSensor" => DeviceType::ContactSensor,
            "Light" => DeviceType::Light,
            "Camera" => DeviceType::Camera,
            _ => {
                return Err(
                    StoreError::DecodeError("Couldn't decode device type".to_string())
                        .into_sqlx_decode_error(),
                )
            }
        };

        Ok(dev_type)
    }
}

