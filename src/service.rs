use std::fs;
use std::path::{Path, PathBuf};

use log::{debug, error, info};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{EmptyResult, OperationResult, OptionalResult};

/// # File cache service
///
/// Supports structs with serde's `Serialize` and `Deserialize` traits.
/// Non thread-safe.
///
/// ## Storage hierarchy:
///
/// `[CACHE BASE DIR]/[INSTANCE NAME]/[NAMESPACE]/[ITEM-NAME]-cache.json`
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

    pub fn store<'a>(&self, namespace: &str, name: &str, item: &impl Serialize) -> EmptyResult {
        info!("store entity '{}' into file cache", name);

        let cache_item_path = self.get_cache_item_path(&self.root_path, &self.instance_name, namespace);

        if !cache_item_path.exists() {
            fs::create_dir_all(&cache_item_path)?;
        }

        debug!("cache item path '{}'", &cache_item_path.display());

        let filename = self.get_filename(name);
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

    pub fn get<'de, T: DeserializeOwned>(&self, namespace: &str, item_name: &str) -> OptionalResult<T> {
        info!("get entity from file cache: namespace='{}', item_name='{}'", namespace, item_name);

        let cache_item_path = self.get_cache_item_path(
            &self.root_path, &self.instance_name, namespace);

        let filename = self.get_filename(item_name);
        let file_path = self.get_cache_file_path(&cache_item_path, &filename);

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
    }

    fn get_cache_item_path(&self, root_path: &str, instance_name: &str, namespace: &str) -> PathBuf {
        Path::new(&root_path).join(&instance_name).join(&namespace)
    }

    fn get_filename(&self, cache_item_name: &str) -> String {
        format!("{}-{}-cache.json", self.instance_name, cache_item_name)
    }

    fn get_cache_file_path(&self, cache_item_path: &PathBuf, cache_item_name: &str) -> PathBuf {
        cache_item_path.join(cache_item_name)
    }
}

#[cfg(test)]
mod get_tests {
    use fake::{Fake, Faker};

    use serde::{Serialize, Deserialize};
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
    use serde::{Serialize, Deserialize};
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

        assert!(service.store(&namespace, &name, &demo).is_ok());

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

        assert!(service.store(&namespace, &name, &demo).is_ok());

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

        assert!(service.store(&namespace, &name, &first_item).is_ok());

        let second_item = Demo {
            login: "Gerry".to_string()
        };

        assert!(service.store(&namespace, &name, &second_item).is_ok());

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