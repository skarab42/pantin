# Pantin

A microservice and libraries collection to take some screenshot with Firefox.

> WIP...

---

## Usage

````shell
Usage: pantin_server [OPTIONS]

Options:
      --server-host <SERVER_HOST>
          Host of the API server [env: PANTIN_SERVER_HOST=] [default: localhost]
  -s, --server-port <SERVER_PORT>
          Port number of the API server [env: PANTIN_SERVER_PORT=] [default: 4242]
  -r, --request-timeout <REQUEST_TIMEOUT>
          Request timeout in seconds [env: PANTIN_REQUEST_TIMEOUT=] [default: 30]
      --browser-pool-max-size <BROWSER_POOL_MAX_SIZE>
          Number of active browser in the pool [env: PANTIN_BROWSER_POOL_MAX_SIZE=] [default: 5]
      --browser-max-age <BROWSER_MAX_AGE>
          Maximum age in seconds of an unused browser session [env: PANTIN_BROWSER_MAX_AGE=] [default: 60]
      --browser-max-recycle-count <BROWSER_MAX_RECYCLE_COUNT>
          Maximum number of times to recycle a browser session [env: PANTIN_BROWSER_MAX_RECYCLE_COUNT=] [default: 10]
      --browser-program <BROWSER_PROGRAM>
          Command or binary path to launch a gecko like browser [env: PANTIN_BROWSER_PROGRAM=] [default: firefox]
      --log-level <LOG_LEVEL>
          Log level [env: PANTIN_LOG_LEVEL=] [default: info] [possible values: info, debug, trace]
  -h, --help
          Print help
  -V, --version
          Print version
````

---

## API Documentation

### Description

Pantin's primary endpoints allow you to:

- Check server availability (`/ping`)
- Request a screenshot of any public webpage (`/screenshot`)

### Endpoints

#### `GET /ping`

- **Purpose**: Health-check endpoint.
- **Response**: Returns JSON with `{ "data": "pong" }`.
- **Example**:
  ```bash
  curl "http://localhost:4242/ping"
  ```
  **Response (JSON):**
  ```json
  { "data": "pong" }
  ```

#### `GET /screenshot`

- **Purpose**: Captures a screenshot of a webpage.
- **Query Parameters**:

| Parameter         | Type     | Default             | Description                                                                                                                               |
|-------------------|----------|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| **url***          | `string` | *none (required)*   | The URL of the page to capture.                                                                                                           |
| **delay**         | `number` | `0`                 | Delay (in ms) after `DOMContentLoaded` before the capture.                                                                                |
| **width**         | `number` | `800`               | Browser window width for the screenshot.                                                                                                  |
| **height**        | `number` | `600`               | Browser window height for the screenshot.                                                                                                 |
| **scrollbar**     | `bool`   | `false`             | Whether to display scrollbars in the screenshot.                                                                                          |
| **mode**          | `string` | `"viewport"`        | Screenshot mode: `"full"`, `"viewport"`, `"selector"`, or `"xpath"`.                                                                      |
| **selector**      | `string` | *none*              | Required if `mode=selector`. A CSS selector for the element to capture.                                                                   |
| **xpath**         | `string` | *none*              | Required if `mode=xpath`. An XPath expression for the element to capture.                                                                 |
| **response_type** | `string` | `"image-png-bytes"` | Output format of the screenshot. Valid options: `attachment`, `image-png-base64`, `image-png-bytes`, `json-png-base64`, `json-png-bytes`. |

- **Example**:
  ```bash
  curl "http://localhost:4242/screenshot?url=https://example.com&mode=full&width=1024&height=768&delay=1000"
  ```

- **Response**:
    - **Status**: 200 OK if successful.
    - **Body** depends on the chosen `response_type`.
        - `image-png-bytes`: Raw PNG bytes, with `Content-Type: image/png`.
        - `attachment`: Raw PNG bytes, but sent as a file attachment (`Content-Disposition`).
        - `image-png-base64`: A data URI string (`data:image/png;base64,...`).
        - `json-png-base64`: A JSON object containing `{ "base64": "..." }`.
        - `json-png-bytes`: A JSON object containing `{ "bytes": [ ... ] }` (PNG data as byte array).

#### Not Found

- **Purpose**: Fallback endpoint for undefined routes.
- **Response**: Returns a 404 JSON error with `{ "cause": "not found" }`.

---

## Running with Docker

```shell 
docker-compose up -d --remove-orphans # start
```

```shell 
docker-compose up -d --build --remove-orphans # start with new build
```

```shell 
docker-compose down --remove-orphans # stop
```

---

## Development

### Set env for development

```shell 
 $env:PANTIN_LOG_LEVEL="trace"
```

### Run for development

```shell 
cargo run
```

### Lint

```shell 
cargo clippy
```

```shell 
cargo clippy --fix # auto fix some rules
```

```shell 
cargo clippy --fix --allow-staged --allow-dirty # Yolo
```

### Format

```shell 
cargo fmt
```

```shell 
cargo test -- --nocapture # show tracing
```

### Build for production

```shell 
cargo build --release
```

### Test

```shell 
cargo test
```

### Code coverage

#### Install [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

```shell
cargo install cargo-llvm-cov
```

#### Run test and collect coverage

```shell
cargo llvm-cov --branch --workspace --html # --open 
```

```shell
open ./target/llvm-cov/html/index.html
```

```shell
cargo llvm-cov clean --workspace # remove artifacts that may affect the coverage results
```

