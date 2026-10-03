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

use std::collections::BTreeSet;
use std::fmt::Display;

use casdoor_rust_sdk::{CasbinRequest, Client, Permission, Role, User};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{Request, Response, State};
use util::abs_path;

#[macro_use]
extern crate rocket;

type ApiResult<T> = Result<Json<T>, (Status, String)>;

fn internal_error(err: impl Display) -> (Status, String) {
    let error_msg = err.to_string();
    eprintln!("{}", error_msg);
    (Status::InternalServerError, error_msg)
}

fn not_found(kind: &str, name: &str) -> (Status, String) {
    (Status::NotFound, format!("{} not found: {}", kind, name))
}

#[get("/login")]
fn login(client: &State<Client>) -> Json<String> {
    Json(client.get_signin_url("http://localhost:8080/callback"))
}

#[get("/signup")]
fn signup(client: &State<Client>) -> Json<String> {
    Json(client.get_signup_url(true, ""))
}

#[get("/auth/<code>")]
async fn callback(client: &State<Client>, code: &str) -> ApiResult<User> {
    let token = client.get_oauth_token(code).await.map_err(internal_error)?;
    let claims = client
        .parse_jwt_token(&token.access_token)
        .map_err(internal_error)?;
    Ok(Json(claims.user))
}

#[get("/logout")]
fn logout() -> Redirect {
    Redirect::to("/")
}

#[get("/user/count/<is_online>")]
async fn user_count(client: &State<Client>, is_online: &str) -> ApiResult<i32> {
    let count = client
        .get_user_count(is_online)
        .await
        .map_err(internal_error)?;
    Ok(Json(count))
}

#[get("/user/<name>")]
async fn get_user(client: &State<Client>, name: &str) -> ApiResult<User> {
    let user = client.get_user(name).await.map_err(internal_error)?;
    user.map(Json).ok_or_else(|| not_found("user", name))
}

#[get("/user/list")]
async fn get_user_list(client: &State<Client>) -> ApiResult<Vec<User>> {
    let users = client.get_users().await.map_err(internal_error)?;
    Ok(Json(users))
}

#[post("/user/delete", data = "<user>")]
async fn delete_user(client: &State<Client>, user: Json<User>) -> ApiResult<bool> {
    let affected = client.delete_user(&user).await.map_err(internal_error)?;
    Ok(Json(affected))
}

#[post("/user/add", data = "<user>")]
async fn add_user(client: &State<Client>, user: Json<User>) -> ApiResult<bool> {
    let affected = client.add_user(&user).await.map_err(internal_error)?;
    Ok(Json(affected))
}

// RBAC with domains (tenants).
//
// In Casdoor, a domain is not an object of its own, it's a name listed in the `domains`
// field of roles and permissions. So creating a domain means adding its name to a role or
// a permission, and deleting it means removing the name from them. The permission's model
// must be an RBAC with domains model, see README.md.

/// List all the domains used by the roles and permissions of the organization.
#[get("/domain/list")]
async fn get_domain_list(client: &State<Client>) -> ApiResult<Vec<String>> {
    let roles = client.get_roles().await.map_err(internal_error)?;
    let permissions = client.get_permissions().await.map_err(internal_error)?;

    let domains: BTreeSet<String> = roles
        .into_iter()
        .flat_map(|role| role.domains)
        .chain(
            permissions
                .into_iter()
                .flat_map(|permission| permission.domains),
        )
        .collect();
    Ok(Json(domains.into_iter().collect()))
}

#[post("/role/<name>/domain/<domain>")]
async fn add_role_domain(client: &State<Client>, name: &str, domain: &str) -> ApiResult<Role> {
    set_role_domain(client, name, domain, true).await
}

#[delete("/role/<name>/domain/<domain>")]
async fn delete_role_domain(client: &State<Client>, name: &str, domain: &str) -> ApiResult<Role> {
    set_role_domain(client, name, domain, false).await
}

async fn set_role_domain(client: &Client, name: &str, domain: &str, add: bool) -> ApiResult<Role> {
    let mut role = client
        .get_role(name)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("role", name))?;

    if set_domain(&mut role.domains, domain, add) {
        client.update_role(&role).await.map_err(internal_error)?;
    }
    Ok(Json(role))
}

#[post("/permission/<name>/domain/<domain>")]
async fn add_permission_domain(
    client: &State<Client>,
    name: &str,
    domain: &str,
) -> ApiResult<Permission> {
    set_permission_domain(client, name, domain, true).await
}

#[delete("/permission/<name>/domain/<domain>")]
async fn delete_permission_domain(
    client: &State<Client>,
    name: &str,
    domain: &str,
) -> ApiResult<Permission> {
    set_permission_domain(client, name, domain, false).await
}

async fn set_permission_domain(
    client: &Client,
    name: &str,
    domain: &str,
    add: bool,
) -> ApiResult<Permission> {
    let mut permission = client
        .get_permission(name)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("permission", name))?;

    if set_domain(&mut permission.domains, domain, add) {
        client
            .update_permission(&permission)
            .await
            .map_err(internal_error)?;
    }
    Ok(Json(permission))
}

/// Add or remove a domain in the list, return whether the list is changed.
fn set_domain(domains: &mut Vec<String>, domain: &str, add: bool) -> bool {
    let exists = domains.iter().any(|d| d == domain);
    if add && !exists {
        domains.push(domain.to_string());
        return true;
    }
    if !add && exists {
        domains.retain(|d| d != domain);
        return true;
    }
    false
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
struct EnforceRequest {
    /// The permission to check against, such as "casbin/permission-1".
    permission_id: String,
    /// The user ID, such as "casbin/alice".
    user: String,
    domain: String,
    resource: String,
    action: String,
}

/// Check whether a user can do an action on a resource in a domain.
#[post("/enforce", data = "<request>")]
async fn enforce(client: &State<Client>, request: Json<EnforceRequest>) -> ApiResult<bool> {
    let casbin_request: CasbinRequest = vec![
        request.user.as_str().into(),
        request.domain.as_str().into(),
        request.resource.as_str().into(),
        request.action.as_str().into(),
    ];

    let allowed = client
        .enforce(&request.permission_id, "", "", "", "", &casbin_request)
        .await
        .map_err(internal_error)?;
    Ok(Json(allowed))
}

#[launch]
fn rocket() -> _ {
    let conf_path = abs_path("conf.toml").expect("Cannot find conf.toml");
    let client = Client::from_toml(&conf_path).expect("Failed to parse conf.toml");

    rocket::build().manage(client).attach(Cors).mount(
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
            get_domain_list,
            add_role_domain,
            delete_role_domain,
            add_permission_domain,
            delete_permission_domain,
            enforce,
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
