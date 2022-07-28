use fake::{Fake, Faker};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Demo {
    pub login: String,
}

pub fn get_demo_entity() -> Demo {
    let login = Faker.fake::<String>();
    Demo { login }
}

pub fn init_env_logging() {
    let _ = env_logger::builder().filter_level(LevelFilter::Debug)
        .is_test(true).try_init();
}