# pantin

A screenshot microservice

WIP...

## Docker

```shell 
docker-compose up -d --build --remove-orphans
```

## Development

### Set env for development

```shell 
 $env:PANTIN_LOG_LEVEL="trace"
```

### Run for development

```shell 
cargo run
```

### Build for production

```shell 
cargo build --release
```

### Lint

```shell 
cargo clippy --fix --allow-staged --allow-dirty
```

### Format

```shell 
cargo fmt
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

