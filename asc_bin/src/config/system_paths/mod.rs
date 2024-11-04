use crate::util;

static QUALIFIER: &str = "";
static ORGANIZATION: &str = "";
static APPLICATION: &str = "asc";

fn build(prefix: &str, name: &str) -> String {
    let path = format!("{prefix}/{name}");
    let dir = &util::fs::get_parent_dir(&path);
    if !util::fs::is_dir_exists(dir) {
        util::fs::create_dir(&dir);
    }
    return path;
}

pub mod conf;
pub use conf::ConfigPath;

pub mod data;
pub use data::DataPath;
