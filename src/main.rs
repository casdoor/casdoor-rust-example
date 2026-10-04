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

#[macro_use]
extern crate rocket;

mod api;
mod auth;
mod error;
mod session;

use std::path::Path;

use casdoor_rust_sdk::Client;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::serde::Deserialize;

use crate::session::Sessions;

/// The settings of this app in Rocket.toml, besides Rocket's own ones such as `port`.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    pub redirect_uri: String,
    pub static_dir: String,
}

#[launch]
fn rocket() -> _ {
    // The Casdoor application this app signs in with, see conf.toml.
    let client = Client::from_toml("conf.toml").expect("failed to read conf.toml");

    let rocket = rocket::build();
    let config: AppConfig = rocket
        .figment()
        .extract()
        .expect("failed to read Rocket.toml");

    // The React app is served by this server too, so the browser talks to one origin
    // and the session cookie just works.
    let rocket = if Path::new(&config.static_dir).is_dir() {
        rocket.mount("/", FileServer::from(&config.static_dir))
    } else {
        rocket.mount("/", routes![not_built])
    };

    rocket
        .manage(client)
        .manage(Sessions::default())
        .manage(config)
        .mount("/", routes![auth::callback])
        .mount(
            "/api",
            routes![
                auth::signin,
                auth::signup,
                auth::account,
                auth::signout,
                api::get_users,
                api::add_user,
                api::delete_user,
                api::get_roles,
                api::get_permissions,
                api::add_role_domain,
                api::delete_role_domain,
                api::add_permission_domain,
                api::delete_permission_domain,
                api::enforce,
            ],
        )
        .register("/api", catchers![error::default_catcher])
}

#[get("/")]
fn not_built() -> RawHtml<&'static str> {
    RawHtml(
        "<p>The React app isn't built yet, run <code>yarn install && yarn build</code> \
         in the <code>web</code> folder and restart the server.</p>",
    )
}
