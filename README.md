<h1 align="center" style="border-bottom: none;">📦⚡️Casdoor Rust Example</h1>
<h3 align="center">An example of casdoor-rust-sdk</h3>

## Architecture

Example contains 2 parts:

| Name     | SDK              | Language         | Source code                                                     |
|----------|------------------|------------------|-----------------------------------------------------------------|
| Frontend | casdoor-vue-sdk  | Javascript + Vue | https://github.com/casdoor/casdoor-rust-example/tree/master/web |
| Backend  | casdoor-rust-sdk | Rust             | https://github.com/casdoor/casdoor-rust-example/                |

## Installation

Example uses Casdoor to manage members. So you need to create an organization and an application for the example in a Casdoor instance.

### Necessary configuration

#### Get the code

```shell
git clone https://github.com/casdoor/casdoor
git clone https://github.com/casdoor/casdoor-rust-example
```

#### Run example

- run casdoor
- configure
- Front end

  ```js
  // in ./web/src/config.js
  export let serverUrl = `http://localhost:5000/api`; // port where rust(backend) runs
  ```

- Back end(conf.toml):

The below config is for the Casdoor demo site: https://door.casdoor.com/, please change it to your own Casdoor instance.

Note: the `certificate` field is omitted as `<...>` due to limited space. For full config, see: https://github.com/casdoor/casdoor-rust-example/blob/master/conf.toml

  ```toml
  endpoint = "https://door.casdoor.com"
  client_id = "294b09fbc17f95daf2fe"
  client_secret = "dd8982f7046ccba1bbd7851d5c1ece4e52bf039d"
  certificate = """-----BEGIN CERTIFICATE-----MIIE+TCCAuGgAwIBAgIDAeJAMA0GCSqGSIb3DQEBCwUAMDYxHTAbBgNVBAoTFENh <...> -----END CERTIFICATE-----"""
  org_name = "casbin"
  ```

- install dependencies

  ```shell
  cd web && yarn install
  ```

- run

  ```bash
  cd web && yarn serve
  cargo run
  ```

Now, example runs its front end at port 8080 and runs it's back end at port 5000. You can modify the code and see what will happen.

## RBAC with domains (tenants)

In Casdoor, a domain is not an object of its own, it's a name listed in the `domains` field of a role or a permission. So the example manages domains by updating roles and permissions, and checks a user's rights in a domain with the `enforce` API. The permission's model must be an RBAC with domains model, for example:

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

| Method | API                                          | Description                                             |
|--------|----------------------------------------------|---------------------------------------------------------|
| GET    | `/api/domain/list`                           | List the domains used by all the roles and permissions  |
| POST   | `/api/role/<name>/domain/<domain>`           | Add a domain to a role                                  |
| DELETE | `/api/role/<name>/domain/<domain>`           | Remove a domain from a role                             |
| POST   | `/api/permission/<name>/domain/<domain>`     | Add a domain to a permission                            |
| DELETE | `/api/permission/<name>/domain/<domain>`     | Remove a domain from a permission                       |
| POST   | `/api/enforce`                               | Check whether a user can do an action in a domain       |

Check a user's right in a domain:

```shell
curl -X POST http://localhost:5000/api/enforce \
  -H "Content-Type: application/json" \
  -d '{"permissionId": "casbin/permission-1", "user": "casbin/alice", "domain": "domain1", "resource": "data1", "action": "read"}'
```

It returns `true` or `false`. The same can be done with the SDK directly:

```rust
let mut role = client.get_role("role-1").await?.unwrap();
role.domains.push("domain1".to_string());
client.update_role(&role).await?;

let allowed = client
    .enforce("casbin/permission-1", "", "", "", "", &vec!["casbin/alice".into(), "domain1".into(), "data1".into(), "read".into()])
    .await?;
```
