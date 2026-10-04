// Copyright 2026 The Casdoor Authors. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use casdoor_rust_sdk::CasdoorError;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::{json, Json};
use rocket::Request;

pub type ApiResult<T> = Result<Json<T>, ApiError>;

/// An API error, it's returned to the browser as `{"error": "..."}`.
#[derive(Debug)]
pub struct ApiError {
    pub status: Status,
    pub message: String,
}

impl ApiError {
    pub fn new(status: Status, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(Status::BadRequest, message)
    }

    pub fn not_found(kind: &str, name: &str) -> Self {
        Self::new(Status::NotFound, format!("{kind} not found: {name}"))
    }
}

impl From<CasdoorError> for ApiError {
    fn from(err: CasdoorError) -> Self {
        Self::new(Status::InternalServerError, err.to_string())
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        if self.status.code >= 500 {
            eprintln!("{} {}: {}", request.method(), request.uri(), self.message);
        }

        (self.status, Json(json!({ "error": self.message }))).respond_to(request)
    }
}

#[catch(default)]
pub fn default_catcher(status: Status, _request: &Request) -> ApiError {
    let message = match status.code {
        401 => "please sign in first",
        403 => "only the admins of the organization can do this",
        _ => status.reason().unwrap_or("unknown error"),
    };

    ApiError::new(status, message)
}
