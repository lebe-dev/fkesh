# fkesh

Synchronous file cache.

Storage format: JSON.

## How to use

Add to `Cargo.toml`:

```toml
[dependencies]
fkesh = { git = "https://github.com/lebe-dev/fkesh.git", version = "0.2.0" }
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
    Some(value) => println!("Chappy is here!"),
    None => eprintln!("Chappy wasn't found :(")
}

```

## Cache live time (TTL)

- `0` - TTL is disabled
- `12345` - TTL in seconds

## Storage: file hierarchy

```
[CACHE-ROOT]/[INSTANCE-NAME]/[NAMESPACE]/[ITEM]-cache.json
[CACHE-ROOT]/[INSTANCE-NAME]/[NAMESPACE]/[ITEM]-cache-metadata.json
```

## What about thread safety, async, etc.?

Async file cache is a huge complex topic and requires a lot of time, 
so I've decided to stay with synchronous implementation. 
This library is suitable for my tiny projects :)