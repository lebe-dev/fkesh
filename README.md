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

let service = FileCacheService::new("/opt/myapp/cache", "demo-instance")?;

let namespace = "demo";

// Entity will be stored in:
// - `/opt/myapp/cache/demo-instance/demo/chappy-cache-metadata.json` file as:
// {
//   "ttl_secs": 10000,
//   "created_unixtime": 1658774583
// }
//
// - `/opt/myapp/cache/demo-instance/demo/chappy-cache.json` file as:
// {
//   "name": "Chappy"
// }
//
service.store(&namespace, "chappy", &chappy, 10000)?;

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
[CACHE-ROOT]/[INSTANCE-NAME]/[NAMESPACE]/[ITEM]-cache-metadata.json
```

## Future plans:

- Thread safe