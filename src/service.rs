use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::error::FileCacheError;
use crate::types::{EmptyResult, OperationResult, OptionalResult};

/// # File cache service
///
/// Supports structs with serde's `Serialize` and `Deserialize` traits.
/// Non thread-safe.
///
/// ## Storage hierarchy:
///
/// Entity file path `[CACHE BASE DIR]/[INSTANCE NAME]/[NAMESPACE]/[ITEM-NAME]-cache.json`
/// Entity metadata-file path `[CACHE BASE DIR]/[INSTANCE NAME]/[NAMESPACE]/[ITEM-NAME]-cache-metadata.json`
///
/// ## Storage format
///
/// Data format: `JSON`
#[derive(Clone)]
pub struct FileCacheService {
    /// Path to cache directory
    root_path: String,

    instance_name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileCacheItemMetadata {
    pub ttl_secs: u64,
    pub created_unixtime: u64
}

pub const CACHE_FILENAME_POSTFIX: &str = "cache.json";
pub const METADATA_FILENAME_POSTFIX: &str = "cache-metadata.json";

impl FileCacheService {

    /// Create instance of FileCacheService
    ///
    /// - `root_path` - root path to cache directory (will be created if doesn't exist)
    /// - `cache_instance_name` - name of current service, included in file hierarchy
    pub fn new(root_path: &str, cache_instance_name: &str) -> OperationResult<FileCacheService> {
        info!("create file cache service, root path '{}', cache name '{}'",
            root_path, cache_instance_name);

        let cache_root_path = Path::new(root_path);

        if !cache_root_path.exists() {
            fs::create_dir_all(cache_root_path)?;
            info!("root path has been created for file cache service '{}'",
                cache_root_path.display());
        }

        Ok(
            FileCacheService {
            root_path: root_path.to_string(),
            instance_name: cache_instance_name.to_string()
        })
    }

    /// Store `item` with cache `name` in `namespace`
    ///
    /// - `ttl_secs` - cache time to live in seconds. `0` - immortal
    pub fn store<'a>(&self, namespace: &str, name: &str, item: &impl Serialize,
                     ttl_secs: u64) -> EmptyResult {

        info!("store entity '{}' into file cache", name);

        let cache_item_path = self.get_cache_item_path(&self.root_path, &self.instance_name, namespace);

        if !cache_item_path.exists() {
            fs::create_dir_all(&cache_item_path)?;
        }

        debug!("cache item path '{}'", &cache_item_path.display());

        let metadata_filename = self.get_filename(
            name, METADATA_FILENAME_POSTFIX);
        let metadata_file_path = self.get_cache_file_path(&cache_item_path,
                                                          &metadata_filename);
        debug!("destination metadata file path '{}'", &metadata_file_path.display());
        let now_unixtime = self.get_now_in_unixtime_secs()?;
        let item_metadata: FileCacheItemMetadata = FileCacheItemMetadata {
            ttl_secs,
            created_unixtime: now_unixtime
        };
        let metadata_json = serde_json::to_string(&item_metadata)?;
        fs::write(&metadata_file_path, metadata_json)?;
        info!("cache item metadata has been created");

        let filename = self.get_filename(name, CACHE_FILENAME_POSTFIX);
        let file_path = self.get_cache_file_path(&cache_item_path, &filename);
        debug!("destination file path '{}'", &file_path.display());

        let json = serde_json::to_string(item)?;

        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        fs::write(&file_path, json)?;

        info!("item '{}' has been saved into file cache", name);
        Ok(())
    }

    /// Get (retrieve) item from cache by `name` and `namespace`
    pub fn get<'de, T: DeserializeOwned>(&self, namespace: &str, item_name: &str) -> OptionalResult<T> {
        info!("get entity from file cache: namespace='{}', item_name='{}'", namespace, item_name);

        let cache_item_path = self.get_cache_item_path(
            &self.root_path, &self.instance_name, namespace);

        let metadata_filename = self.get_filename(
            item_name, METADATA_FILENAME_POSTFIX);
        let metadata_file_path = self.get_cache_file_path(&cache_item_path,
                                                          &metadata_filename);
        debug!("destination metadata file path '{}'", &metadata_file_path.display());

        let filename = self.get_filename(item_name, CACHE_FILENAME_POSTFIX);
        let file_path = self.get_cache_file_path(&cache_item_path, &filename);

        if metadata_file_path.exists() {
            let metadata_json = fs::read_to_string(&metadata_file_path)?;
            let metadata = serde_json::from_str::<FileCacheItemMetadata>(&metadata_json)?;

            let now_unixtime = self.get_now_in_unixtime_secs()?;

            if now_unixtime > metadata.created_unixtime {
                let diff_secs = now_unixtime - metadata.created_unixtime;

                if metadata.ttl_secs > 0 && (diff_secs > metadata.ttl_secs) {
                    info!("cache item '{}' has been expired and will be removed", item_name);

                    if file_path.exists() {
                        fs::remove_file(file_path)?;
                        fs::remove_file(metadata_file_path)?;
                    }

                    return Ok(None)
                }

            }

            if file_path.exists() {
                let json = fs::read_to_string(&file_path)?;

                match serde_json::from_str::<T>(&json) {
                    Ok(value) => {
                        info!("entity '{}' has been loaded from file cache", item_name);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        error!("couldn't deserialize cache item: {}", e);
                        fs::remove_file(&file_path)?;
                        Ok(None)
                    }
                }

            } else {
                info!("file cache entity '{}' wasn't found", item_name);
                Ok(None)
            }

        } else {
            info!("metadata file not found for item '{}', cache file will be removed", item_name);
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
            Ok(None)
        }
    }

    fn get_cache_item_path(&self, root_path: &str, instance_name: &str, namespace: &str) -> PathBuf {
        Path::new(&root_path).join(&instance_name).join(&namespace)
    }

    fn get_filename(&self, cache_item_name: &str, postfix: &str) -> String {
        format!("{}-{}", cache_item_name, postfix)
    }

    fn get_cache_file_path(&self, cache_item_path: &PathBuf, cache_item_name: &str) -> PathBuf {
        cache_item_path.join(cache_item_name)
    }

    fn get_now_in_unixtime_secs(&self) -> OperationResult<u64> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(tm) => Ok(tm.as_secs()),
            Err(e) => {
                error!("{}", e);
                Err(FileCacheError::Default)
            }
        }
    }
}

#[cfg(test)]
mod ttl_tests {
    use std::fs;
    use std::path::Path;
    use std::thread::sleep;
    use std::time::Duration;

    use fake::{Fake, Faker};
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use crate::service::{CACHE_FILENAME_POSTFIX, FileCacheService, METADATA_FILENAME_POSTFIX};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Demo {
        pub login: String
    }

    #[test]
    fn delete_all_cache_item_file_if_metadata_is_missing() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 1000).is_ok());

        let metadata_filename = format!("{}-{}", &name, METADATA_FILENAME_POSTFIX);
        let metadata_file = Path::new(&root_path_str)
            .join(&instance_name)
            .join(&namespace)
            .join(metadata_filename);

        fs::remove_file(metadata_file).unwrap();

        assert!(service.get::<Demo>(&namespace, &name).unwrap().is_none());

        let cache_item_filename = format!("{}-{}", &name, CACHE_FILENAME_POSTFIX);
        let cache_item_file = Path::new(&root_path_str)
            .join(&instance_name)
            .join(&namespace)
            .join(cache_item_filename);

        assert!(!cache_item_file.exists());
    }

    #[test]
    fn return_none_if_metadata_companion_file_is_missing() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 1000).is_ok());

        let metadata_filename = format!("{}-{}", &name, METADATA_FILENAME_POSTFIX);
        let metadata_file = Path::new(&root_path_str)
            .join(&instance_name)
            .join(&namespace)
            .join(metadata_filename);

        fs::remove_file(metadata_file).unwrap();

        assert!(service.get::<Demo>(&namespace, &name).unwrap().is_none());
    }

    #[test]
    fn return_item_with_existing_ttl() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 1000).is_ok());

        sleep(Duration::from_secs(1));

        let result = service.get::<Demo>(&namespace, &name).unwrap().unwrap();

        assert_eq!(result, demo);
    }

    #[test]
    fn return_none_for_item_with_expired_ttl() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 1).is_ok());

        sleep(Duration::from_secs(3));

        assert!(service.get::<Demo>(&namespace, &name).unwrap().is_none());
    }

    #[test]
    fn remove_files_for_cache_item_with_expired_ttl() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 1).is_ok());

        sleep(Duration::from_secs(3));

        assert!(service.get::<Demo>(&namespace, &name).unwrap().is_none());

        let metadata_filename = format!("{}-{}", &name, METADATA_FILENAME_POSTFIX);
        let metadata_file = Path::new(&root_path_str)
            .join(&instance_name)
            .join(&namespace)
            .join(metadata_filename);

        assert!(!metadata_file.exists());

        let cache_item_filename = format!("{}-{}", &name, CACHE_FILENAME_POSTFIX);
        let cache_item_file = Path::new(&root_path_str)
            .join(&instance_name)
            .join(&namespace)
            .join(cache_item_filename);

        assert!(!cache_item_file.exists());
    }

    #[test]
    fn item_should_be_retrieved_with_zero_ttl() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 0).is_ok());

        sleep(Duration::from_secs(1));

        let result = service.get::<Demo>(&namespace, &name).unwrap().unwrap();

        assert_eq!(result, demo);
    }

    fn get_demo_entity() -> Demo {
        Demo {  login: "Barry".to_string() }
    }
}

#[cfg(test)]
mod get_tests {
    use fake::{Fake, Faker};
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use crate::service::FileCacheService;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Demo {
        pub login: String
    }

    #[test]
    fn return_none_for_unknown_cache_item() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        assert!(service.get::<Demo>(&namespace, &name).unwrap().is_none());
    }
}

#[cfg(test)]
mod store_tests {
    use std::path::Path;

    use fake::{Fake, Faker};
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use crate::service::FileCacheService;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Demo {
        pub login: String
    }

    #[test]
    fn store_and_get() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 0).is_ok());

        let result = service.get::<Demo>(&namespace, &name).unwrap().unwrap();

        assert_eq!(result, demo);
    }

    #[test]
    fn directory_hierarchy_should_be_created() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let demo = get_demo_entity();

        assert!(service.store(&namespace, &name, &demo, 0).is_ok());

        assert!(
            Path::new(&root_path_str)
                .join(instance_name)
                .join(namespace)
                .exists()
        );
    }

    #[test]
    fn previous_cache_item_file_should_be_overwritten() {
        let root_path_tmp = tempdir().unwrap();
        let root_path = root_path_tmp.path();
        let root_path_str = format!("{}", root_path.display());

        let instance_name = Faker.fake::<String>();

        let service = FileCacheService::new(
            &root_path_str, &instance_name).unwrap();

        let namespace = Faker.fake::<String>();
        let name = Faker.fake::<String>();

        let first_item = get_demo_entity();

        assert!(service.store(&namespace, &name, &first_item, 0).is_ok());

        let second_item = Demo {
            login: "Gerry".to_string()
        };

        assert!(service.store(&namespace, &name, &second_item, 0).is_ok());

        assert!(
            Path::new(&root_path_str)
                .join(instance_name)
                .join(namespace)
                .exists()
        );
    }

    fn get_demo_entity() -> Demo {
        Demo {  login: "Jerry".to_string() }
    }
}

#[cfg(test)]
mod new_tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::service::FileCacheService;

    #[test]
    fn create_root_path_if_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let root_path = tmp_dir.path();

        fs::remove_dir(root_path).unwrap();

        assert!(!root_path.exists());

        let root_path_str = format!("{}", root_path.display());

        FileCacheService::new(&root_path_str, "whatever").unwrap();

        assert!(root_path.exists());
    }
}