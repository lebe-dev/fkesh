# fkesh

Non thread-safe file cache.

Storage format: JSON.

## How to use

Add to `Cargo.toml`:

```toml
[dependencies]
fkesh = { git = "https://github.com/lebe-dev/fkesh.git", version = "0.1.0" }
```

Use:

```rust
let service = FileCacheService::new("path/to/cache", "demo")?;
```

## Storage: file hierarchy

```
[CACHE-ROOT]/[INSTANCE-NAME]/[NAMESPACE]/[ITEM]-cache.json
```