// Copyright 2022 The Casdoor Authors. All Rights Reserved.
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

mod util;

use casdoor_rust_sdk::{AuthService, CasdoorConfig};
use casdoor_rust_sdk::{CasdoorUser, UserService};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::tokio::task;
use rocket::{Request, Response};
use util::abs_path;

#[macro_use]
extern crate rocket;

#[get("/login")]
fn login() -> Json<String> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let auth_service = AuthService::new(&conf);

    let redirect_url = auth_service.get_signin_url("http://localhost:8080/callback".to_string());
    Json(redirect_url)
}

#[get("/signup")]
fn signup() -> Json<String> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let auth_service = AuthService::new(&conf);

    let redirect_url = auth_service.get_signup_url_enable_password();
    Json(redirect_url)
}

#[get("/auth/<code>")]
async fn callback(code: String) -> Json<CasdoorUser> {
    let user = task::spawn_blocking(move || {
        // perform the computation
        let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
        let auth_service = AuthService::new(&conf);

        let token = auth_service.get_auth_token(code).unwrap();

        let user = auth_service
            .parse_jwt_token(token)
            .expect("parse jwt token failed");

        user
    })
    .await
    .unwrap();

    Json(user)
}

#[get("/logout")]
fn logout() -> Redirect {
    Redirect::to("/")
}

#[get("/user/count/<is_online>")]
async fn user_count(is_online: String) -> Json<i64> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let count = user_service.get_user_count(is_online).await.unwrap();
    Json(count)
}

#[get("/user/<name>")]
async fn get_user(name: String) -> Json<CasdoorUser> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let user = user_service.get_user(name).await.unwrap();
    Json(user)
}

#[get("/user/list")]
async fn get_user_list() -> Json<Vec<CasdoorUser>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let users = user_service.get_users().await.unwrap();
    Json(users)
}

#[post("/user/delete", data = "<user>")]
async fn delete_user(user: Json<CasdoorUser>) -> Json<u16> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let code = user_service.delete_user(user.0).await.unwrap();
    Json(code.as_u16())
}

#[post("/user/add", data = "<user>")]
async fn add_user(user: Json<CasdoorUser>) -> Json<u16> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let code = user_service.add_user(user.0).await.unwrap();
    Json(code.as_u16())
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(Cors).mount(
        "/api",
        routes![
            login,
            signup,
            callback,
            logout,
            user_count,
            get_user,
            get_user_list,
            delete_user,
            add_user,
        ],
    )
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
