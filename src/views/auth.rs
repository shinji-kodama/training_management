use serde::{Deserialize, Serialize};

use crate::models::_entities::users;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub pid: String,
    pub name: String,
    pub is_verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionLoginResponse {
    pub success: bool,
    pub user: SessionUserInfo,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionUserInfo {
    pub pid: String,
    pub name: String,
    pub email: String,
    pub is_verified: bool,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: &users::Model, token: &String) -> Self {
        Self {
            token: token.to_string(),
            pid: user.pid.to_string(),
            name: user.name.clone(),
            is_verified: user.email_verified_at.is_some(),
        }
    }
}

impl SessionLoginResponse {
    #[must_use]
    pub fn new(user: &users::Model, csrf_token: String) -> Self {
        Self {
            success: true,
            user: SessionUserInfo {
                pid: user.pid.to_string(),
                name: user.name.clone(),
                email: user.email.clone(),
                is_verified: user.email_verified_at.is_some(),
            },
            csrf_token,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentResponse {
    pub pid: String,
    pub name: String,
    pub email: String,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
        }
    }
}
