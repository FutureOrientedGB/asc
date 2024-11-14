use std::collections::BTreeSet;

use crate::{
    config::{self, project::ProjectConfig, relative_paths},
    errors::ErrorTag,
    util,
};

use super::ConfigType;

use clap::Args;

#[derive(Args, Debug, Default, Clone)]
pub struct RunArgs {
    name: Option<String>,

    args: Option<Vec<String>>,

    #[clap(long, default_value = ConfigType::Debug.as_ref())]
    config: ConfigType,
}

impl RunArgs {
    pub fn exec(&self) -> bool {
        tracing::info!(message = "run");

        if let Some(project_conf) = config::project::ProjectConfig::read_project_conf() {
            if let Some(workspace) = project_conf.workspace {
                let cwd = util::fs::get_cwd();
                let mut flat_project_conf = config::project::ProjectConfig::default();
                let mut flat_bins = BTreeSet::new();
                for member in workspace.members {
                    util::fs::set_cwd(&member);
                    if let Some(pc) = config::project::ProjectConfig::read_project_conf() {
                        for bin in &pc.bins {
                            flat_bins.insert(config::project::EntryConfig {
                                name: bin.name.clone(),
                                ..Default::default()
                            });
                        }
                    }
                    util::fs::set_cwd(&cwd);
                }
                flat_project_conf.bins = flat_bins;
                return self.run_bin(&flat_project_conf);
            } else {
                return self.run_bin(&project_conf);
            }
        }

        return false;
    }

    fn run_bin(&self, project_conf: &ProjectConfig) -> bool {
        let mut bin_name = String::new();
        let mut bin_names = vec![];
        if project_conf.bins.len() == 1 {
            bin_name = project_conf.bins.first().unwrap().name.clone();
            bin_names.push(bin_name.clone());
        } else {
            for bin in &project_conf.bins {
                bin_names.push(bin.name.clone());
                if let Some(n) = &self.name {
                    if &bin.name == n {
                        bin_name = bin.name.clone();
                        break;
                    }
                }
            }
        }
        if bin_name.is_empty() {
            tracing::error!(
                error_tag = ErrorTag::InvalidCliArgsError.as_ref(),
                bins = bin_names.join(", ")
            );
            return false;
        }
        return util::shell::run(
            &format!(
                "{}/{}/{}/{}",
                relative_paths::ASC_TARGET_DIR_NAME,
                bin_name,
                self.config.as_ref(),
                bin_name
            ),
            &self
                .args
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|s| s.as_str())
                .collect(),
            false,
            false,
            false,
        )
        .is_ok();
    }
}
