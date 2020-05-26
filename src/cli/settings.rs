use clap::ArgMatches;
use log::*;

pub struct Settings {
    setup: Option<String>,
    env: Option<String>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            setup: None,
            env: None,
        }
    }

    pub fn set_setup(&mut self, setup: String) {
        self.setup = Some(setup);
    }

    pub fn set_env(&mut self, env: String) {
        self.env = Some(env)
    }

    pub fn setup(&self) -> Option<&String> {
        self.setup.as_ref()
    }

    pub fn env(&self) -> Option<&String> {
        self.env.as_ref()
    }
}

pub fn get_settings(app: &ArgMatches) -> Settings {
    let mut settings = Settings::new();
    if let Some(setup) = app.value_of_lossy("setup") {
        settings.set_setup(setup.to_string());
    }
    info!("setup {:?}", settings.setup());
    if let Some(env) = app.value_of_lossy("environment") {
        settings.set_env(env.to_string());
    }
    info!("env {:?}", settings.env());
    settings
}

#[cfg(test)]
mod test {

    use crate::cli::settings::Settings;

    #[test]
    fn settings() {
        let mut s = Settings::new();
        assert_eq!(s.setup(), None);
        assert_eq!(s.env(), None);
        s.set_setup("setup".to_string());
        s.set_env("env".to_string());
        assert_eq!(Some(&"setup".to_string()), s.setup());
        assert_eq!(Some(&"env".to_string()), s.env());
    }
}