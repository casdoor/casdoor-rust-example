<h1 align="center" style="border-bottom: none;">📦⚡️Casdoor rust example</h1>
<h3 align="center">An example of casdoor-rust-sdk</h3>

## Architecture

Example contains 2 parts:

| Name     | SDK              | Language         | Source code                                                     |
| -------- | ---------------- | ---------------- | --------------------------------------------------------------- |
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

  ```toml
  # all demo data, plz replace them to yourself's data.

  endpoint = "http://localhost:8000"
  client_id = "client_id"
  client_secret = "client_secret"
  jwt_pub_key = """jwt_pub_key"""
  org_name = "built-in"
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
