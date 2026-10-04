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

//! The APIs that call Casdoor as the application, with the client ID and secret in
//! conf.toml. Reading needs a signed-in user, changing data needs an admin.

use casdoor_rust_sdk::{CasbinRequest, Client, Permission, Role, User};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

use crate::error::{ApiError, ApiResult};
use crate::session::{Admin, SignedIn};

#[get("/users")]
pub async fn get_users(client: &State<Client>, _user: SignedIn) -> ApiResult<Vec<User>> {
    Ok(Json(client.get_users().await?))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct NewUser {
    name: String,
    display_name: String,
    password: String,
}

#[post("/users", data = "<new_user>")]
pub async fn add_user(
    client: &State<Client>,
    _admin: Admin,
    new_user: Json<NewUser>,
) -> Result<Status, ApiError> {
    let user = User {
        owner: client.config.organization_name.clone(),
        name: new_user.name.clone(),
        display_name: new_user.display_name.clone(),
        password: new_user.password.clone(),
        ..Default::default()
    };

    client.add_user(&user).await?;
    Ok(Status::Created)
}

#[delete("/users/<name>")]
pub async fn delete_user(
    client: &State<Client>,
    admin: Admin,
    name: &str,
) -> Result<Status, ApiError> {
    if admin.0.name == name && admin.0.owner == client.config.organization_name {
        return Err(ApiError::bad_request("you can't delete yourself"));
    }

    let user = client
        .get_user(name)
        .await?
        .ok_or_else(|| ApiError::not_found("user", name))?;
    client.delete_user(&user).await?;
    Ok(Status::NoContent)
}

// RBAC with domains (tenants).
//
// In Casdoor, a domain is not an object of its own, it's a name in the `domains` field of
// roles and permissions. Creating a domain means adding its name to a role or a
// permission, and deleting it means removing the name from them.

#[get("/roles")]
pub async fn get_roles(client: &State<Client>, _user: SignedIn) -> ApiResult<Vec<Role>> {
    Ok(Json(client.get_roles().await?))
}

#[get("/permissions")]
pub async fn get_permissions(
    client: &State<Client>,
    _user: SignedIn,
) -> ApiResult<Vec<Permission>> {
    Ok(Json(client.get_permissions().await?))
}

#[post("/roles/<name>/domains/<domain>")]
pub async fn add_role_domain(
    client: &State<Client>,
    _admin: Admin,
    name: &str,
    domain: &str,
) -> ApiResult<Role> {
    update_role_domains(client, name, |domains| add_domain(domains, domain)).await
}

#[delete("/roles/<name>/domains/<domain>")]
pub async fn delete_role_domain(
    client: &State<Client>,
    _admin: Admin,
    name: &str,
    domain: &str,
) -> ApiResult<Role> {
    update_role_domains(client, name, |domains| domains.retain(|d| d != domain)).await
}

async fn update_role_domains(
    client: &Client,
    name: &str,
    update: impl FnOnce(&mut Vec<String>),
) -> ApiResult<Role> {
    let mut role = client
        .get_role(name)
        .await?
        .ok_or_else(|| ApiError::not_found("role", name))?;

    update(&mut role.domains);
    client.update_role(&role).await?;
    Ok(Json(role))
}

#[post("/permissions/<name>/domains/<domain>")]
pub async fn add_permission_domain(
    client: &State<Client>,
    _admin: Admin,
    name: &str,
    domain: &str,
) -> ApiResult<Permission> {
    update_permission_domains(client, name, |domains| add_domain(domains, domain)).await
}

#[delete("/permissions/<name>/domains/<domain>")]
pub async fn delete_permission_domain(
    client: &State<Client>,
    _admin: Admin,
    name: &str,
    domain: &str,
) -> ApiResult<Permission> {
    update_permission_domains(client, name, |domains| domains.retain(|d| d != domain)).await
}

async fn update_permission_domains(
    client: &Client,
    name: &str,
    update: impl FnOnce(&mut Vec<String>),
) -> ApiResult<Permission> {
    let mut permission = client
        .get_permission(name)
        .await?
        .ok_or_else(|| ApiError::not_found("permission", name))?;

    update(&mut permission.domains);
    client.update_permission(&permission).await?;
    Ok(Json(permission))
}

fn add_domain(domains: &mut Vec<String>, domain: &str) {
    if !domains.iter().any(|d| d == domain) {
        domains.push(domain.to_string());
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct EnforceRequest {
    /// The permission to check against, such as "my-org/permission-1".
    permission_id: String,
    /// The user ID such as "my-org/alice", the signed-in user if it's empty.
    #[serde(default)]
    user: String,
    domain: String,
    resource: String,
    action: String,
}

/// Check whether a user can do an action on a resource in a domain.
#[post("/enforce", data = "<request>")]
pub async fn enforce(
    client: &State<Client>,
    user: SignedIn,
    request: Json<EnforceRequest>,
) -> ApiResult<bool> {
    let subject = if request.user.is_empty() {
        user.0.get_id()
    } else {
        request.user.clone()
    };

    let casbin_request: CasbinRequest = vec![
        subject.into(),
        request.domain.as_str().into(),
        request.resource.as_str().into(),
        request.action.as_str().into(),
    ];

    let allowed = client
        .enforce(&request.permission_id, "", "", "", "", &casbin_request)
        .await?;
    Ok(Json(allowed))
}
