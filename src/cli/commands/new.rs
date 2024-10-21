use clap::Args;

use handlebars::Handlebars;

use serde_json;

use super::init;
use crate::errors::ErrorTag;
use crate::{
    cli::{config, template},
    util,
};

#[derive(Args, Debug, Clone, Default)]
pub struct NewArgs {
    pub name: Option<String>,
    #[clap(long, default_value_t = false)]
    pub lib: bool,
    #[clap(long, default_value_t = false)]
    pub workspace: bool,
    pub member: Option<Vec<String>>,
}

impl NewArgs {
    pub fn exec(&self) -> bool {
        if self.name.is_some() {
            if self.workspace && self.member.is_some() {
                return self.new_workspace();
            } else if !self.lib {
                return self.new_bin(self.name.as_ref().unwrap());
            } else {
                return self.new_lib(self.name.as_ref().unwrap());
            }
        }
        return false;
    }

    fn new_bin(&self, name: &str) -> bool {
        tracing::info!("new bin");
        // write asc.toml
        if !self.new_package(name) {
            return false;
        }

        // write main.cpp
        return std::fs::write(
            format!(
                "{}/{}/{}",
                name,
                config::PROJECT_SRC_DIR,
                config::PROJECT_BIN_SRC
            ),
            template::NEW_BIN_HBS.as_bytes(),
        )
        .is_ok();
    }

    fn new_lib(&self, name: &str) -> bool {
        tracing::info!("new lib");
        // write asc.toml
        if !self.new_package(name) {
            return false;
        }

        {
            // write export.h
            let reg = Handlebars::new();
            match reg.render_template(
                template::NEW_LIB_EXPORT_HBS,
                &serde_json::json!({"project_upper": name.to_uppercase()}),
            ) {
                Err(e) => {
                    tracing::error!(
                        call = "Handlebars::render_template",
                        template = template::NEW_LIB_EXPORT_HBS,
                        error_tag = ErrorTag::RenderHandlebarsError.as_ref(),
                        error_str = e.to_string()
                    );

                    return false;
                }
                Ok(text) => {
                    let path = format!(
                        "{}/{}/{}",
                        name,
                        config::PROJECT_SRC_DIR,
                        config::PROJECT_EXPORT_SRC
                    );
                    if let Err(e) = std::fs::write(&path, text.as_bytes()) {
                        tracing::error!(
                            call = "std::fs::write",
                            path = path,
                            error_tag = ErrorTag::WriteFileError.as_ref(),
                            error_str = e.to_string(),
                            message = text,
                        );
                        return false;
                    }
                }
            }
        }

        {
            // write main.cpp
            let reg = Handlebars::new();
            match reg.render_template(
                template::NEW_LIB_MAIN_HBS,
                &serde_json::json!({"project_upper": name.to_uppercase()}),
            ) {
                Err(e) => {
                    tracing::error!(
                        call = "Handlebars::render_template",
                        template = template::NEW_LIB_EXPORT_HBS,
                        error_tag = ErrorTag::RenderHandlebarsError.as_ref(),
                        error_str = e.to_string()
                    );

                    return false;
                }
                Ok(text) => {
                    let path = format!(
                        "{}/{}/{}",
                        name,
                        config::PROJECT_SRC_DIR,
                        config::PROJECT_LIB_SRC
                    );
                    if let Err(e) = std::fs::write(&path, text.as_bytes()) {
                        tracing::error!(
                            call = "std::fs::write",
                            path = path,
                            error_tag = ErrorTag::WriteFileError.as_ref(),
                            error_str = e.to_string(),
                            message = text,
                        );
                        return false;
                    }
                }
            }
        }

        return true;
    }

    fn new_package(&self, name: &str) -> bool {
        tracing::info!("new package");
        // validate args
        if name.is_empty() {
            tracing::error!(
                call = "name.is_empty",
                error_tag = ErrorTag::InvalidCliArgsError.as_ref(),
            );
            return false;
        }

        // skip is exists
        if util::fs::is_file_exists(name) {
            tracing::error!(
                call = "util::fs::is_file_exists",
                path = name,
                error_tag = ErrorTag::FileExistsError.as_ref()
            );
            return false;
        }

        // create src dir
        let src_dir = format!("{name}/{}", config::PROJECT_SRC_DIR);
        if let Err(e) = std::fs::create_dir_all(&src_dir) {
            tracing::error!(
                call = "std::fs::create_dir_all",
                path = src_dir,
                error_tag = ErrorTag::CretaeDirectoryError.as_ref(),
                error_str = e.to_string()
            );
            return false;
        }

        let cwd = util::fs::get_cwd();

        // init
        util::fs::set_cwd(name);
        let mut args = init::InitArgs::default();
        args.lib = self.lib;
        args.workspace = self.workspace;
        args.member = self.member.clone();
        return args.init_package(name) && util::fs::set_cwd(&cwd);
    }

    fn new_workspace(&self) -> bool {
        // validate args
        let name = self.name.as_ref().unwrap();
        let members = self.member.as_ref().unwrap();
        if name.is_empty() || members.is_empty() {
            return false;
        }

        // skip is exists
        if util::fs::is_file_exists(name) {
            tracing::error!(
                call = "util::fs::is_file_exists",
                path = name,
                error_tag = ErrorTag::FileExistsError.as_ref()
            );
            return false;
        }

        let cwd = util::fs::get_cwd();

        if let Err(e) = std::fs::create_dir(name) {
            tracing::info!(
                call = "std::fs::create_dir",
                path = name,
                error_tag = e.to_string()
            );
            return false;
        }

        // create members
        util::fs::set_cwd(name);
        let mut has_error = false;
        let mut workspace = config::WorkSpaceConfig::default();
        for m in members {
            if workspace.members.insert(m.clone()) {
                if self.lib {
                    if !self.new_lib(m) {
                        has_error = true;
                    }
                } else {
                    if !self.new_bin(m) {
                        has_error = true;
                    }
                }
            }
        }
        let mut project = config::ProjectConfig::default();
        project.workspace = Some(workspace);

        util::fs::set_cwd(&cwd);

        // skip if exists
        let path = format!("{name}/{}", config::PROJECT_TOML);
        if util::fs::is_file_exists(&path) {
            tracing::error!(
                call = "util::fs::is_file_exists",
                path = config::PROJECT_TOML,
                error_tag = ErrorTag::FileExistsError.as_ref(),
            );
            return false;
        }

        // write asc.toml
        return !has_error && project.validate() && project.dump(&path);
    }
}
