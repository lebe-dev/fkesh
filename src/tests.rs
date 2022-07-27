use fake::{Fake, Faker};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Demo {
    pub login: String,
}

pub fn get_demo_entity() -> Demo {
    let login = Faker.fake::<String>();
    Demo { login }
}