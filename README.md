# Casdoor Rust Example

An example web app that uses [Casdoor](https://casdoor.org) for sign-in and authorization through [casdoor-rust-sdk](https://github.com/casdoor/casdoor-rust-sdk). It shows how to:

- sign users in and up with Casdoor (OAuth 2.0 authorization code flow)
- read and manage users
- manage domains (tenants) and check a user's rights in a domain (RBAC with domains)

| Part     | Stack                          | Port | Source        |
|----------|--------------------------------|------|---------------|
| Backend  | Rust, Rocket, casdoor-rust-sdk | 5000 | [`src/`](src) |
| Frontend | Vue 3                          | 8080 | [`web/`](web) |

## Quick start

You need Rust (stable), Node.js and Yarn. The repo is configured for the Casdoor demo site [door.casdoor.com](https://door.casdoor.com), so it runs without any setup:

```shell
git clone https://github.com/casdoor/casdoor-rust-example
cd casdoor-rust-example
cargo run
```

In another terminal:

```shell
cd web
yarn install
yarn serve
```

Open http://localhost:8080 and click **Sign in**. After signing in on Casdoor you are redirected back and the home page shows your user info.

## Use your own Casdoor

1. [Install Casdoor](https://casdoor.org/docs/basic/server-installation) and sign in as admin.
2. Create an organization and an application in it. Add `http://localhost:8080/callback` to the application's **Redirect URLs**.
3. Fill in [`conf.toml`](conf.toml) with the application's settings:

   ```toml
   endpoint = "http://localhost:8000"            # Casdoor server URL
   client_id = "<client ID of the application>"
   client_secret = "<client secret of the application>"
   certificate = """-----BEGIN CERTIFICATE-----
   <the application's certificate, see Certs in Casdoor>
   -----END CERTIFICATE-----"""
   org_name = "<organization name>"
   app_name = "<application name>"
   ```

The backend reads `conf.toml` from the current directory, so run `cargo run` from the repo root. The backend URL used by the frontend is set in [`web/src/config.js`](web/src/config.js).

## How sign-in works

1. The frontend calls `GET /api/login`, the backend returns the Casdoor sign-in URL (`client.get_signin_url()`), and the browser goes there.
2. After signing in, Casdoor redirects to `http://localhost:8080/callback?code=...`.
3. The frontend sends the code to `GET /api/auth/<code>`. The backend exchanges it for an access token (`client.get_oauth_token()`), verifies the token with the certificate and returns the user (`client.parse_jwt_token()`).
4. The frontend keeps the user in `localStorage` and shows it on `/home`.

## Backend APIs

All APIs are under `http://localhost:5000/api`. Errors are returned as plain text with status 404 or 500.

| Method | Path                                    | Description                                              |
|--------|-----------------------------------------|----------------------------------------------------------|
| GET    | `/login`                                | Casdoor sign-in URL                                      |
| GET    | `/signup`                               | Casdoor sign-up URL                                      |
| GET    | `/auth/<code>`                          | Exchange an authorization code for the signed-in user    |
| GET    | `/user/list`                            | All users of the organization                            |
| GET    | `/user/<name>`                          | One user                                                 |
| GET    | `/user/count/<is_online>`               | Number of online (`1`) or offline (`0`) users            |
| POST   | `/user/add`                             | Add a user (JSON body), returns `true` if added          |
| POST   | `/user/delete`                          | Delete a user (JSON body), returns `true` if deleted     |
| GET    | `/domain/list`                          | Domains used by the organization's roles and permissions |
| POST   | `/role/<name>/domain/<domain>`          | Add a domain to a role                                   |
| DELETE | `/role/<name>/domain/<domain>`          | Remove a domain from a role                              |
| POST   | `/permission/<name>/domain/<domain>`    | Add a domain to a permission                             |
| DELETE | `/permission/<name>/domain/<domain>`    | Remove a domain from a permission                        |
| POST   | `/enforce`                              | Check whether a user can do an action in a domain        |

The APIs other than sign-in are called with the application's client ID and secret, so the application must belong to the organization it manages. The demo site's application doesn't, so `/enforce` and the write APIs need your own Casdoor.

## RBAC with domains

A domain (tenant) lets one user have different roles in different places, e.g. admin in `domain1` but only a reader in `domain2`. In Casdoor a domain is not a separate object, it's a name in the **Domains** field of a role or a permission:

- creating a domain = adding its name to a role or permission
- deleting a domain = removing it from them
- checking a right = calling enforce with `[user, domain, resource, action]`

To try it in your own Casdoor:

1. Create a model with domains:

   ```ini
   [request_definition]
   r = sub, dom, obj, act

   [policy_definition]
   p = sub, dom, obj, act

   [role_definition]
   g = _, _, _

   [policy_effect]
   e = some(where (p.eft == allow))

   [matchers]
   m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act
   ```

2. Create a role (e.g. `role-1`) with some users, and a permission (e.g. `permission-1`) that uses the model, the role, some resources and actions.
3. Add a domain to both:

   ```shell
   curl -X POST http://localhost:5000/api/role/role-1/domain/domain1
   curl -X POST http://localhost:5000/api/permission/permission-1/domain/domain1
   ```

4. Check a user's right in the domain, the result is `true` or `false`:

   ```shell
   curl -X POST http://localhost:5000/api/enforce \
     -H "Content-Type: application/json" \
     -d '{"permissionId": "my-org/permission-1", "user": "my-org/alice", "domain": "domain1", "resource": "data1", "action": "read"}'
   ```

The same with the SDK directly:

```rust
use casdoor_rust_sdk::Client;

let client = Client::from_toml("conf.toml")?;

let mut role = client.get_role("role-1").await?.unwrap();
role.domains.push("domain1".to_string());
client.update_role(&role).await?;

let request = vec!["my-org/alice".into(), "domain1".into(), "data1".into(), "read".into()];
let allowed = client.enforce("my-org/permission-1", "", "", "", "", &request).await?;
```

See [`src/main.rs`](src/main.rs) for the full code.

## Links

- [casdoor-rust-sdk](https://github.com/casdoor/casdoor-rust-sdk) ([docs.rs](https://docs.rs/casdoor-rust-sdk))
- [Casdoor docs](https://casdoor.org/docs/overview)
- [Casdoor permissions](https://casdoor.org/docs/permission/overview)

## License

[Apache-2.0](LICENSE)
