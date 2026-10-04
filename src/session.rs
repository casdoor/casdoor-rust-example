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

use std::collections::HashMap;
use std::sync::Mutex;

use casdoor_rust_sdk::User;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::request::{FromRequest, Outcome, Request};
use uuid::Uuid;

const SESSION_COOKIE: &str = "session_id";

/// A signed-in user and the access token Casdoor issued to them.
pub struct Session {
    pub user: User,
    pub access_token: String,
}

/// The sessions of the signed-in users, keyed by a random session ID that's kept in an
/// HttpOnly cookie. They are kept in memory, so restarting the server signs everyone
/// out; a real app would keep them in a database or Redis.
#[derive(Default)]
pub struct Sessions(Mutex<HashMap<String, Session>>);

impl Sessions {
    /// Save a new session and set its ID in the session cookie.
    pub fn create(&self, cookies: &CookieJar<'_>, session: Session) {
        let id = Uuid::new_v4().to_string();
        self.0.lock().unwrap().insert(id.clone(), session);

        cookies.add(
            Cookie::build((SESSION_COOKIE, id))
                .http_only(true)
                .same_site(SameSite::Lax),
        );
    }

    /// Remove the session of the session cookie, and the cookie itself.
    pub fn remove(&self, cookies: &CookieJar<'_>) -> Option<Session> {
        let id = cookies.get(SESSION_COOKIE)?.value().to_string();
        cookies.remove(SESSION_COOKIE);
        self.0.lock().unwrap().remove(&id)
    }

    fn get_user(&self, cookies: &CookieJar<'_>) -> Option<User> {
        let id = cookies.get(SESSION_COOKIE)?.value();
        let sessions = self.0.lock().unwrap();
        sessions.get(id).map(|session| session.user.clone())
    }
}

/// A request guard for the APIs that need a signed-in user, it fails with 401 otherwise.
pub struct SignedIn(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SignedIn {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        let sessions = request.rocket().state::<Sessions>().unwrap();

        match sessions.get_user(request.cookies()) {
            Some(user) => Outcome::Success(SignedIn(user)),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

/// A request guard for the APIs that change data, it needs an admin of the organization
/// (or a global admin of the built-in organization) and fails with 403 otherwise.
pub struct Admin(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        let SignedIn(user) = match request.guard::<SignedIn>().await {
            Outcome::Success(signed_in) => signed_in,
            Outcome::Error(error) => return Outcome::Error(error),
            Outcome::Forward(status) => return Outcome::Forward(status),
        };

        if is_admin(&user) {
            Outcome::Success(Admin(user))
        } else {
            Outcome::Error((Status::Forbidden, ()))
        }
    }
}

pub fn is_admin(user: &User) -> bool {
    user.is_admin || user.owner == "built-in"
}
