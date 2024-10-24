use crate::{config, errors::ErrorTag, util};

use super::ConfigType;

use clap::Args;

#[derive(Args, Debug, Default, Clone)]
pub struct RunArgs {
    name: Option<String>,

    args: Option<Vec<String>>,

    #[clap(long)]
    config: ConfigType,
}

impl RunArgs {
    pub fn exec(&self) -> bool {
        tracing::info!(message = "run");

        if let Some(project_conf) = config::project::ProjectConfig::read_project_conf() {
            if let Some(workspace) = project_conf.workspace {
                if let Some(name) = &self.name {
                    return util::shell::run(
                        &format!(
                            "{}/{}/{}/{}",
                            config::project::path::PROJECT_TARGET_DIR,
                            name,
                            self.config.as_ref(),
                            name
                        ),
                        &self
                            .args
                            .as_ref()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|s| s.as_str())
                            .collect(),
                        None,
                        None,
                    )
                    .is_ok();
                } else {
                    tracing::error!(
                        error_tag = ErrorTag::InvalidCliArgsError.as_ref(),
                        members = workspace.get_members()
                    );
                }
            }
            if let Some(package) = project_conf.package {
                return util::shell::run(
                    &format!(
                        "{}/{}/{}",
                        config::project::path::PROJECT_TARGET_DIR,
                        self.config.as_ref(),
                        package.name
                    ),
                    &self
                        .args
                        .as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                    None,
                    None,
                )
                .is_ok();
            }
        }

        return false;
    }
}
