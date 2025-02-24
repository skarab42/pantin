# pantin

A screenshot microservice

WIP...

## Docker

```shell 
docker-compose up -d --build --remove-orphans
```

## Set env for development

```shell 
 $env:PANTIN_LOG_LEVEL="trace"
```

## Run for development

```shell 
cargo run
```

## Build for production

```shell 
cargo build --release
```

## Lint

```shell 
cargo clippy --fix --allow-staged --allow-dirty
```

## Format

```shell 
cargo fmt
```