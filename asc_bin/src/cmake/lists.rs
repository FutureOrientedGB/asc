use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::Datelike;

use handlebars::Handlebars;

use serde::{Deserialize, Serialize};

pub use super::path;
pub use super::template;
use crate::clang;
use crate::cli;
use crate::config;
use crate::util;

#[derive(Default, Debug, Deserialize, Serialize,)]
struct SourcesGroup {
    dir: String,
    original_dir: String,
    variable: String,
    files: Vec<String>,
}

#[derive(Default, Debug, Deserialize, Serialize,)]
struct InstallHeader {
    src: String,
    dst: String,
}

#[derive(Default, Debug, Deserialize, Serialize,)]
struct CMakeListsData {
    cmake_version: String,
    is_workspace: bool,
    project: String,
    project_upper: String,
    build_year: i32,
    build_month: u32,
    build_day: u32,
    user_cmake_txt: String,
    install_bin_dir: String,
    install_lib_dir: String,
    install_include_dir: String,
    install_share_dir: String,
    executable: bool,
    library: bool,
    shared_library: bool,
    sources_group_by_dir: Vec<SourcesGroup>,
    include_dirs: Vec<String>,
    link_libraries: bool,
    link_public_libraries: bool,
    public_libraries: Vec<String>,
    link_private_libraries: bool,
    private_libraries: Vec<String>,
    install_headers: Vec<InstallHeader>,
}

pub fn gen(
    options: &cli::commands::scan::ScanOptions,
    source_mappings: &clang::parser::SourceMappings,
    is_workspace: bool,
) {
    // output default config.in.cm if not exists
    if !util::fs::is_file_exists(path::CONFIG_H_CM_PATH) {
        std::fs::write(
            path::CONFIG_H_CM_PATH,
            template::CONFIG_IN_CM_HBS.as_bytes(),
        )
        .unwrap();
    }

    // output default user.cmake if not exists
    if !util::fs::is_file_exists(path::USER_CMAKE_PATH) {
        std::fs::write(
            path::USER_CMAKE_PATH,
            template::USER_CMAKE_HBS.as_bytes(),
        )
        .unwrap()
    }

    // group data
    let (group_sources, classify_to_dir, install_headers) = group_data(options, source_mappings);

    // init data
    let local_date_time = chrono::prelude::Local::now();
    let mut data = CMakeListsData::default();
    data.is_workspace = is_workspace;
    data.cmake_version = options.cmake_minimum_version.clone();
    data.project = options.project.clone();
    data.project_upper = options.project.to_uppercase();
    data.build_year = local_date_time.year();
    data.build_month = local_date_time.month();
    data.build_day = local_date_time.day();
    data.user_cmake_txt =
        std::fs::read_to_string(path::USER_CMAKE_PATH).unwrap_or(String::new());
    data.install_bin_dir = config::project::path::INSTALL_BIN_DIR.to_string();
    data.install_lib_dir = config::project::path::INSTALL_LIB_DIR.to_string();
    data.install_include_dir = config::project::path::INSTALL_INCLUDE_DIR.to_string();
    data.install_share_dir = config::project::path::INSTALL_SHARE_DIR.to_string();
    data.executable = !options.static_lib && !options.shared_lib;
    data.library = options.static_lib || options.shared_lib;
    data.shared_library = data.library && options.shared_lib;
    data.include_dirs = options.include_dirs.clone();
    data.link_libraries = false;
    data.link_public_libraries = false;
    data.link_private_libraries = false;

    for (dir, sources) in &group_sources {
        let mut group = SourcesGroup::default();
        group.dir = dir.clone();
        group.original_dir = classify_to_dir.get(dir).unwrap().clone();
        group.variable = format!("${}{}{}", "{", dir, "}");
        for src in sources {
            group.files.push(src.clone());
        }
        data.sources_group_by_dir.push(group);
    }

    for (src, dst) in install_headers {
        data.install_headers
            .push(InstallHeader { src: src, dst: dst });
    }

    // render template
    {
        // write project-config.cmake.in
        let reg = Handlebars::new();
        let text = reg
            .render_template(template::CMAKE_CONFIG_HBS, &data)
            .unwrap();
        std::fs::write(path::config_cmake_in_path(&options.project), text.as_bytes()).unwrap();
    }

    {
        // write version.h.in
        let reg = Handlebars::new();
        let text = reg
            .render_template(template::VERSION_IN_HBS, &data)
            .unwrap();
        std::fs::write(path::VERSION_H_IN_PATH, text.as_bytes()).unwrap();
    }

    {
        // write CMakeLists.txt
        let reg = Handlebars::new();
        let text = reg
            .render_template(template::CMAKE_LISTS_HBS, &data)
            .unwrap();
        std::fs::write(path::CMAKE_LISTS_PATH, text.as_bytes()).unwrap();
    }
}

pub fn gen_workspace(cmake_minimum_version: &str, project: &str, members: &Vec<String>) {
    let data = serde_json::json!({
        "cmake_version": cmake_minimum_version,
        "project": project,
        "members": members,
    });

    // write CMakeLists.txt
    let reg = Handlebars::new();
    let text = reg
        .render_template(template::CMKAE_WORKSPACE_HBS, &data)
        .unwrap();
    std::fs::write(
        path::CMAKE_LISTS_PATH,
        text.as_bytes(),
    )
    .unwrap();
}

fn group_data(
    options: &cli::commands::scan::ScanOptions,
    source_mappings: &clang::parser::SourceMappings,
) -> (
    BTreeMap<String, BTreeSet<String>>,
    HashMap<String, String>,
    BTreeMap<String, String>,
) {
    // group sources by dir name
    let mut group_sources = BTreeMap::<String, BTreeSet<String>>::new();
    let mut classify_to_dir = HashMap::<String, String>::new();
    let mut install_headers = BTreeMap::<String, String>::new();
    for (header, sources) in &source_mappings.header_include_by_sources {
        {
            let header_locate_dir = std::path::Path::new(header)
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            // prepare install headers's src and dst
            let src = if header.starts_with(&options.source_dir) {
                util::fs::remove_prefix(header, &options.project_dir, &options.target_dir)
            } else {
                format!(
                    "${{CMAKE_CURRENT_BINARY_DIR}}/{}",
                    util::fs::remove_prefix(header, &options.source_dir, &options.target_dir)
                )
            };

            let dst = if header_locate_dir.starts_with(&options.source_dir) {
                util::fs::remove_prefix(&header_locate_dir, &options.source_dir, &options.target_dir)
            } else {
                String::new()
            };
            install_headers.insert(src, dst);
        }

        {
            // group header
            let relative_path: String =
                util::fs::remove_prefix(header, &options.project_dir, &options.target_dir);
            let dir = std::path::Path::new(&relative_path)
                .parent()
                .unwrap()
                .to_str()
                .unwrap();
            let classify = dir.replace("/", "_");
            classify_to_dir.insert(classify.clone(), dir.to_string());
            group_sources
                .entry(classify.to_string())
                .or_default()
                .insert(relative_path);
        }

        {
            for src in sources {
                // group source
                let relative_path: String =
                    util::fs::remove_prefix(src, &options.project_dir, &options.target_dir);
                let dir = std::path::Path::new(&relative_path)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let classify = dir.replace("/", "_");
                classify_to_dir.insert(classify.clone(), dir.to_string());
                group_sources
                    .entry(classify.to_string())
                    .or_default()
                    .insert(relative_path);
            }
        }
    }

    return (group_sources, classify_to_dir, install_headers);
}
