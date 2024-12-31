use config::{Config, ConfigError, Environment};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    db_name: String,
    db_path: String,
}

impl Settings {
    #[allow(dead_code)]
    fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(Environment::with_prefix("TRAINER"))
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use rstest::{fixture, rstest};
    use std::env;
    use tempfile::{tempdir, TempDir};

    #[fixture]
    fn db_name() -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        format!("testdb-{}.db3", rand_string)
    }

    #[fixture]
    fn temp_dir() -> TempDir {
        tempdir().unwrap()
    }

    #[rstest]
    fn test_environment_config(db_name: String, temp_dir: TempDir) {
        env::set_var("TRAINER_DB_NAME", db_name.clone());
        env::set_var("TRAINER_DB_PATH", temp_dir.path().as_os_str());

        let setting_result = Settings::new();
        assert!(setting_result.is_ok());

        let settings = setting_result.unwrap();
        assert_eq!(db_name, settings.db_name);
        assert_eq!(
            temp_dir.path().as_os_str().to_str().unwrap(),
            settings.db_path
        );
    }
}
