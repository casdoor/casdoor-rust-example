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

//! Sign-in with the OAuth 2.0 authorization code flow:
//!
//! 1. `/api/signin` redirects the browser to the Casdoor sign-in page.
//! 2. After signing in, Casdoor redirects the browser to `/callback` with a code.
//! 3. `/callback` exchanges the code for an access token, reads the user from the token
//!    and saves them in a session.

use casdoor_rust_sdk::{Client, User};
use rocket::http::{Cookie, CookieJar, RawStr, SameSite, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::time::Duration;
use rocket::State;
use uuid::Uuid;

use crate::error::ApiError;
use crate::session::{Session, Sessions, SignedIn};
use crate::AppConfig;

const STATE_COOKIE: &str = "oauth_state";

#[get("/signin")]
pub fn signin(
    client: &State<Client>,
    config: &State<AppConfig>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let url = client.get_signin_url(&config.redirect_uri);
    Redirect::to(with_random_state(&url, cookies))
}

/// Same as `/signin`, but opens the sign-up page. A new user is signed in right after
/// signing up, so it ends at `/callback` too.
#[get("/signup")]
pub fn signup(
    client: &State<Client>,
    config: &State<AppConfig>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let url = client.get_signup_url(false, &config.redirect_uri);
    Redirect::to(with_random_state(&url, cookies))
}

/// Replace the `state` parameter of a Casdoor sign-in URL with a random value, and keep
/// the value in a cookie. `/callback` only accepts the state in the cookie, so another
/// site can't sign the browser in with its own code (login CSRF).
fn with_random_state(url: &str, cookies: &CookieJar<'_>) -> String {
    let state = Uuid::new_v4().simple().to_string();
    cookies.add(
        Cookie::build((STATE_COOKIE, state.clone()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::minutes(10)),
    );

    let url = url.split_once("&state=").map_or(url, |(url, _)| url);
    format!("{url}&state={state}")
}

/// Casdoor redirects here after sign-in. Errors are passed to the React app as
/// `/?error=...` so that it can show them.
#[get("/callback?<code>&<state>&<error>")]
pub async fn callback(
    client: &State<Client>,
    sessions: &State<Sessions>,
    cookies: &CookieJar<'_>,
    code: Option<&str>,
    state: Option<&str>,
    error: Option<&str>,
) -> Redirect {
    match sign_in(client, sessions, cookies, code, state, error).await {
        Ok(()) => Redirect::to("/"),
        Err(err) => {
            eprintln!("Sign-in failed: {}", err.message);
            let message = RawStr::new(&err.message).percent_encode();
            Redirect::to(format!("/?error={message}"))
        }
    }
}

async fn sign_in(
    client: &Client,
    sessions: &Sessions,
    cookies: &CookieJar<'_>,
    code: Option<&str>,
    state: Option<&str>,
    error: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(error) = error {
        return Err(ApiError::bad_request(error));
    }

    let expected_state = cookies.get(STATE_COOKIE).map(|c| c.value().to_string());
    cookies.remove(STATE_COOKIE);
    if state.is_none() || state != expected_state.as_deref() {
        return Err(ApiError::bad_request(
            "the sign-in has expired or wasn't started here, please sign in again",
        ));
    }

    let code = code.ok_or_else(|| ApiError::bad_request("no code is returned by Casdoor"))?;
    let token = client.get_oauth_token(code).await?;
    // The access token is a JWT signed by Casdoor, verifying it with the certificate in
    // conf.toml proves that the user in it is real.
    let claims = client.parse_jwt_token(&token.access_token)?;

    sessions.create(
        cookies,
        Session {
            user: claims.user,
            access_token: token.access_token,
        },
    );
    Ok(())
}

/// The signed-in user.
#[get("/account")]
pub fn account(user: SignedIn) -> Json<User> {
    Json(user.0)
}

/// Sign out of this app and end the session in Casdoor too, so that the next sign-in asks
/// for the password again.
#[post("/signout")]
pub async fn signout(
    client: &State<Client>,
    sessions: &State<Sessions>,
    cookies: &CookieJar<'_>,
) -> Status {
    if let Some(session) = sessions.remove(cookies) {
        if let Err(err) = client.logout_current_session(&session.access_token).await {
            eprintln!("Failed to sign out of Casdoor: {err}");
        }
    }

    Status::NoContent
}
