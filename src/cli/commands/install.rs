use clap::Args;

use super::{scan::ScanOptions, ConfigType};
use crate::{cmake, config, util};

#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    #[clap(long, default_value = "target/installed")]
    pub prefix: String,
    #[clap(long, default_value = "debug")]
    config: ConfigType,
}

impl InstallArgs {
    pub fn exec(&self) -> bool {
        tracing::info!(message = "install", name = util::fs::get_cwd_name());

        if !config::ProjectConfig::is_project_inited(false) {
            return false;
        }

        if !config::ProjectConfig::is_source_scaned() {
            return false;
        }

        let options = ScanOptions {
            target_dir: config::path::PROJECT_TARGET_DIR.to_string(),
            cmake_config: self.config.as_ref().to_string(),
            ..Default::default()
        };
        cmake::install::exec(&options, &self.prefix);

        return true;
    }
}
