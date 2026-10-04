# Casdoor Rust Example

An example web app that uses [Casdoor](https://casdoor.org) for sign-in and authorization through [casdoor-rust-sdk](https://github.com/casdoor/casdoor-rust-sdk). It shows how to:

- sign users in and up with Casdoor (OAuth 2.0 authorization code flow) and keep them in a session
- list, add and delete the users of an organization
- manage domains (tenants) and check a user's rights in a domain (RBAC with domains)

The backend is a [Rocket](https://rocket.rs) server in [`src/`](src), the frontend is a React app in [`web/`](web). The server also serves the built React app, so everything runs on one port: http://localhost:8080.

## Quick start

You need Rust (stable) and Node.js 20.19+ with Yarn. The repo is configured for the Casdoor demo site [door.casdoor.com](https://door.casdoor.com), so it runs without any setup:

```shell
git clone https://github.com/casdoor/casdoor-rust-example
cd casdoor-rust-example
cd web && yarn install && yarn build && cd ..
cargo run
```

Open http://localhost:8080 and click **Sign in**.

## Use your own Casdoor

1. [Install Casdoor](https://casdoor.org/docs/basic/server-installation) and sign in as admin.
2. Create an organization and an application in it. Add `http://localhost:8080/callback` to the application's **Redirect URLs**.
3. Fill in [`conf.toml`](conf.toml) with the application's settings:

   ```toml
   endpoint = "http://localhost:8000"            # Casdoor server URL
   client_id = "<client ID of the application>"
   client_secret = "<client secret of the application>"
   certificate = """-----BEGIN CERTIFICATE-----
   <the certificate of the application, see Certs in Casdoor>
   -----END CERTIFICATE-----"""
   org_name = "<organization name>"
   app_name = "<application name>"
   ```

The port, the callback URL and the folder of the React app are set in [`Rocket.toml`](Rocket.toml). Run `cargo run` from the repo root, since `conf.toml` and `Rocket.toml` are read from the current folder.

## How it works

Sign-in uses the OAuth 2.0 authorization code flow, see [`src/auth.rs`](src/auth.rs):

1. **Sign in** opens `/api/signin`, which redirects the browser to the Casdoor sign-in page. A random `state` is kept in a cookie to stop login CSRF.
2. After signing in, Casdoor redirects the browser to `/callback?code=...&state=...`.
3. The server checks the `state`, exchanges the code for an access token (`client.get_oauth_token()`) and verifies the token with the certificate (`client.parse_jwt_token()`).
4. The user is kept in a server-side session, the browser only gets a random session ID in an HttpOnly cookie. The React app reads the user from `/api/account`.
5. **Sign out** deletes the session and ends the user's session in Casdoor too (`client.logout_current_session()`).

The other APIs call Casdoor as the application, with the client ID and secret in `conf.toml`, see [`src/api.rs`](src/api.rs). They need a signed-in user, and the ones that change data need an admin of the organization. Otherwise they return 401 or 403.

Sessions are kept in memory to keep the example short, so restarting the server signs everyone out. A real app would keep them in a database or Redis.

## APIs

| Method | Path                                       | Who    | Description                                       |
|--------|--------------------------------------------|--------|---------------------------------------------------|
| GET    | `/api/signin`                              | anyone | Redirect to the Casdoor sign-in page              |
| GET    | `/api/signup`                              | anyone | Redirect to the Casdoor sign-up page              |
| GET    | `/callback`                                | anyone | Casdoor redirects here after sign-in              |
| GET    | `/api/account`                             | user   | The signed-in user                                |
| POST   | `/api/signout`                             | anyone | Sign out of this app and of Casdoor               |
| GET    | `/api/users`                               | user   | The users of the organization                     |
| POST   | `/api/users`                               | admin  | Add a user: `{"name", "displayName", "password"}` |
| DELETE | `/api/users/<name>`                        | admin  | Delete a user                                     |
| GET    | `/api/roles`                               | user   | The roles of the organization                     |
| GET    | `/api/permissions`                         | user   | The permissions of the organization               |
| POST   | `/api/roles/<name>/domains/<domain>`       | admin  | Add a domain to a role                            |
| DELETE | `/api/roles/<name>/domains/<domain>`       | admin  | Remove a domain from a role                       |
| POST   | `/api/permissions/<name>/domains/<domain>` | admin  | Add a domain to a permission                      |
| DELETE | `/api/permissions/<name>/domains/<domain>` | admin  | Remove a domain from a permission                 |
| POST   | `/api/enforce`                             | user   | Check whether a user can do an action in a domain |

Errors are returned as `{"error": "..."}`.

## RBAC with domains

A domain (tenant) lets one user have different rights in different places, e.g. a reader in `domain1` but nothing in `domain2`. In Casdoor a domain is not a separate object, it's a name in the **Domains** field of a role or a permission:

- creating a domain = adding its name to a role or permission
- deleting a domain = removing it from them
- checking a right = calling enforce with `[user, domain, resource, action]`

To try it, in your own Casdoor:

1. Create a model with a domain in its request:

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

2. Create a role with some users and the domain `domain1`.
3. Create a permission with the model above, **Resource type** `Custom` (Casdoor only allows 3-field models for the `Application` type), the role, the domain `domain1`, and some resources and actions, e.g. `data1` and `read`.
4. Sign in to the example as an admin of the organization and open **Domains**. You can add and remove domains of the roles and permissions there, and check a right, e.g. whether `my-org/alice` can `read` `data1` in `domain1`.

The same with curl, using the session cookie of a signed-in browser:

```shell
curl -X POST http://localhost:8080/api/enforce \
  -H "Content-Type: application/json" \
  -H "Cookie: session_id=<session ID>" \
  -d '{"permissionId": "my-org/permission-1", "user": "my-org/alice", "domain": "domain1", "resource": "data1", "action": "read"}'
```

And with the SDK directly:

```rust
use casdoor_rust_sdk::Client;

let client = Client::from_toml("conf.toml")?;

let mut role = client.get_role("role-1").await?.unwrap();
role.domains.push("domain2".to_string());
client.update_role(&role).await?;

let request = vec!["my-org/alice".into(), "domain1".into(), "data1".into(), "read".into()];
let allowed = client.enforce("my-org/permission-1", "", "", "", "", &request).await?;
```

The demo site doesn't allow its application to call enforce or change data, so those only work with your own Casdoor.

## Develop the frontend

`yarn dev` serves the React app at http://localhost:5173 with hot reload, and forwards `/api` and `/callback` to the Rust server. Start the server with the callback URL of the dev server, and add that URL to the application's Redirect URLs in Casdoor:

```shell
ROCKET_REDIRECT_URI=http://localhost:5173/callback cargo run
```

```shell
cd web
yarn dev
```

## Links

- [casdoor-rust-sdk](https://github.com/casdoor/casdoor-rust-sdk) ([docs.rs](https://docs.rs/casdoor-rust-sdk))
- [Casdoor docs](https://casdoor.org/docs/overview)
- [Casdoor permissions](https://casdoor.org/docs/permission/overview)

## License

[Apache-2.0](LICENSE)
