use std::{env, error::Error, net::SocketAddr, path::PathBuf};

/// Runtime configuration. Defaults suit local development; override with env vars.
#[derive(Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_path: PathBuf,
    pub web_dir: PathBuf, // starter:web
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            database_path: "data/kudamerah.db".into(),
            web_dir: crate::web::DEFAULT_DIR.into(), // starter:web
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let mut config = Self::default();
        if let Ok(addr) = env::var("BIND_ADDR") {
            config.bind_addr = addr.parse()?;
        }
        if let Some(path) = env::var_os("DATABASE_PATH") {
            config.database_path = path.into();
        }
        // starter:web:begin
        if let Some(dir) = env::var_os("WEB_DIR") {
            config.web_dir = dir.into();
        }
        // starter:web:end
        Ok(config)
    }
}
