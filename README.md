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
struct Dog {
    pub name: String
}

let chappy = Dog {
    name: "Chappy".to_string();
};

let service = FileCacheService::new("path/to/cache", "demo-instance")?;

let namespace = "demo";

service.store(&namespace, "chappy", &chappy)?;

match service.get(&namespace, "chappy")? {
    Some(value) => {
        println!("Chappy is here!")
    }
    None => {
        eprintln!("Chappy wasn't found :(")
    }
}

```

## Storage: file hierarchy

```
[CACHE-ROOT]/[INSTANCE-NAME]/[NAMESPACE]/[ITEM]-cache.json
```