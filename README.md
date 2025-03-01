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

#### Install [tarpaulin](https://crates.io/crates/cargo-tarpaulin)

```shell
cargo install cargo-tarpaulin
```

#### Run test and collect coverage

```shell
cargo tarpaulin --all-features --workspace --engine llvm --out html
```

```shell
open ./tarpaulin-report.html
```
