use color_eyre::eyre::{Context, Result};
use figment::providers::{Env, Format, Json, Toml, Yaml};
use figment::Figment;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub enum WhichTerms {
    All,
    MostRecentOnly,
    #[serde(untagged)]
    These(Vec<String>),
}

impl Default for WhichTerms {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_gradescope_base_url")]
    pub gradescope_base_url: Url,
    pub gradescope_cookie: String,

    #[serde(default)]
    pub which_terms: WhichTerms,
}

fn default_gradescope_base_url() -> Url {
    "https://www.gradescope.com".parse().unwrap()
}

pub fn read_config() -> Result<Config> {
    Figment::new()
        .merge(Env::raw())
        .merge(Json::file("config.json"))
        .merge(Yaml::file("config.yaml"))
        .merge(Toml::file("config.toml"))
        .extract()
        .wrap_err("Failed to parse config")
}
