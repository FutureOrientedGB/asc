use super::ConfigType;

use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
    #[clap(long, default_value = "Debug")]
    config: ConfigType,
}


impl BuildArgs {
    pub fn exec(&self) -> bool {
        false
    }
}