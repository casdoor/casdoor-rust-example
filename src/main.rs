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

use casdoor_rust_sdk::{
    ApplicationService, CasdoorApplication, CasdoorCert, CasdoorGroup, CasdoorOrganization,
    CasdoorPermission, CasdoorResource, CasdoorRole, CasdoorUser, CertService, GroupService,
    OrganizationService, PermissionService, ResourceService, RoleService, UserService,
};
use casdoor_rust_sdk::{AuthService, CasdoorConfig};
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
fn login() -> Result<Json<String>, String> {
    let path = abs_path("conf.toml").map_err(|err| {
        let error_msg = format!("Error getting the absolute path of conf.toml: {:?}", err);
        eprintln!("{}", &error_msg);
        error_msg
    })?;

    let conf = CasdoorConfig::from_toml(&path).map_err(|err| {
        let error_msg = format!("Error parsing the configuration file: {:?}", err);
        eprintln!("{}", &error_msg);
        error_msg
    })?;

    let auth_service = AuthService::new(&conf);
    let redirect_url = auth_service.get_signin_url("http://localhost:8080/callback".to_string());
    Ok(Json(redirect_url))
}

#[get("/signup")]
fn signup() -> Json<String> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let auth_service = AuthService::new(&conf);

    let redirect_url = auth_service.get_signup_url_enable_password();
    Json(redirect_url)
}

#[get("/auth/<code>")]
async fn callback(code: String) -> Result<Json<CasdoorUser>, String> {
    let user_result = task::spawn_blocking(move || {
        let conf_path = abs_path("conf.toml").map_err(|_| {
            let err_msg = "Cannot find conf.toml".to_string();
            eprintln!("abs_path() error: {}", err_msg);
            err_msg
        })?;
        let conf_str = conf_path.as_str();
        let conf = CasdoorConfig::from_toml(conf_str).map_err(|_| {
            let err_msg = "Failed to parse TOML config".to_string();
            eprintln!("from_toml() error: {}", err_msg);
            err_msg
        })?;

        let auth_service = AuthService::new(&conf);
        let token = auth_service.get_auth_token(code).map_err(|e| {
            let err_msg = e.to_string();
            eprintln!("get_auth_token() error: {}", err_msg);
            err_msg
        })?;
        let user = auth_service.parse_jwt_token(token).map_err(|e| {
            let err_msg = e.to_string();
            eprintln!("parse_jwt_token() error: {}", err_msg);
            err_msg
        })?;

        Ok(user)
    })
    .await
    .map_err(|_| {
        let err_msg = "Failed to process in spawn_blocking".to_string();
        eprintln!("{}", err_msg);
        err_msg
    })?;

    match user_result {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e),
    }
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

#[post("/user/update", data = "<user>")]
async fn update_user(user: Json<CasdoorUser>) -> Json<u16> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let user_service = UserService::new(&conf);

    let code = user_service.update_user(user.0).await.unwrap();
    Json(code.as_u16())
}

// Organization routes
#[get("/organization/list")]
async fn get_organization_list() -> Json<Vec<CasdoorOrganization>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let organization_service = OrganizationService::new(&conf);

    let organizations = organization_service.get_organizations().await.unwrap();
    Json(organizations)
}

#[get("/organization/<name>")]
async fn get_organization(name: String) -> Json<CasdoorOrganization> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let organization_service = OrganizationService::new(&conf);

    let organization = organization_service.get_organization(name).await.unwrap();
    Json(organization)
}

// Application routes
#[get("/application/list")]
async fn get_application_list() -> Json<Vec<CasdoorApplication>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let application_service = ApplicationService::new(&conf);

    let applications = application_service.get_applications().await.unwrap();
    Json(applications)
}

#[get("/application/<name>")]
async fn get_application(name: String) -> Json<CasdoorApplication> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let application_service = ApplicationService::new(&conf);

    let application = application_service.get_application(name).await.unwrap();
    Json(application)
}

// Group routes
#[get("/group/list")]
async fn get_group_list() -> Json<Vec<CasdoorGroup>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let group_service = GroupService::new(&conf);

    let groups = group_service.get_groups().await.unwrap();
    Json(groups)
}

#[get("/group/<name>")]
async fn get_group(name: String) -> Json<CasdoorGroup> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let group_service = GroupService::new(&conf);

    let group = group_service.get_group(name).await.unwrap();
    Json(group)
}

// Role routes
#[get("/role/list")]
async fn get_role_list() -> Json<Vec<CasdoorRole>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let role_service = RoleService::new(&conf);

    let roles = role_service.get_roles().await.unwrap();
    Json(roles)
}

#[get("/role/<name>")]
async fn get_role(name: String) -> Json<CasdoorRole> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let role_service = RoleService::new(&conf);

    let role = role_service.get_role(name).await.unwrap();
    Json(role)
}

// Permission routes
#[get("/permission/list")]
async fn get_permission_list() -> Json<Vec<CasdoorPermission>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let permission_service = PermissionService::new(&conf);

    let permissions = permission_service.get_permissions().await.unwrap();
    Json(permissions)
}

#[get("/permission/<name>")]
async fn get_permission(name: String) -> Json<CasdoorPermission> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let permission_service = PermissionService::new(&conf);

    let permission = permission_service.get_permission(name).await.unwrap();
    Json(permission)
}

// Resource routes
#[get("/resource/list")]
async fn get_resource_list() -> Json<Vec<CasdoorResource>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let resource_service = ResourceService::new(&conf);

    let resources = resource_service.get_resources().await.unwrap();
    Json(resources)
}

#[get("/resource/<name>")]
async fn get_resource(name: String) -> Json<CasdoorResource> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let resource_service = ResourceService::new(&conf);

    let resource = resource_service.get_resource(name).await.unwrap();
    Json(resource)
}

// Cert routes
#[get("/cert/list")]
async fn get_cert_list() -> Json<Vec<CasdoorCert>> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let cert_service = CertService::new(&conf);

    let certs = cert_service.get_certs().await.unwrap();
    Json(certs)
}

#[get("/cert/<name>")]
async fn get_cert(name: String) -> Json<CasdoorCert> {
    let conf = CasdoorConfig::from_toml(abs_path("conf.toml").unwrap().as_str()).unwrap();
    let cert_service = CertService::new(&conf);

    let cert = cert_service.get_cert(name).await.unwrap();
    Json(cert)
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
            update_user,
            get_organization_list,
            get_organization,
            get_application_list,
            get_application,
            get_group_list,
            get_group,
            get_role_list,
            get_role,
            get_permission_list,
            get_permission,
            get_resource_list,
            get_resource,
            get_cert_list,
            get_cert,
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
