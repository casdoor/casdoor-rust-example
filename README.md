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
