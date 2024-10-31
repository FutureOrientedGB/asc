use std::collections::HashMap;

use basic_trie::DataTrie;
use config_file_derives::ConfigFile;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    cli::commands::VcpkgArgs,
    config::{self, relative_paths::VCPKG_PORTS_DIR_NAME},
    errors::ErrorTag,
    util,
};

use super::VcpkgManager;

// from vcpkg (git log)
#[derive(Clone, Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("toml")]
pub struct GitCommitInfo {
    #[serde(skip)]
    pub path: String,

    pub hash: String,
    pub date_time: String,
}

// from vcpkg (versions/baseline.json)
#[derive(Clone, Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
pub struct VcpkgBaseline {
    #[serde(skip)]
    path: String,

    pub default: HashMap<String, VcpkgPortVersion>,
}

// asc
#[derive(Clone, Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
pub struct VcpkgSearchIndex {
    #[serde(skip)]
    path: String,

    pub prefix_trie: DataTrie<String>,
    pub postfix_trie: DataTrie<String>,

    pub baseline: VcpkgBaseline,

    check_point: GitCommitInfo,
}

// from vcpkg
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct VcpkgPortVersion {
    pub baseline: String,
    pub port_version: u32,
}

impl VcpkgPortVersion {
    pub fn format_version_text(&self) -> String {
        if self.port_version == 0 {
            self.baseline.clone()
        } else {
            format!("{}#{}", self.baseline, self.port_version)
        }
    }
}

// from vcpkg
#[derive(Clone, Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
pub struct VcpkgPortVersions {
    #[serde(skip)]
    path: String,

    versions: Vec<VcpkgPortTreeVersion>,
}

// from vcpkg
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct VcpkgPortTreeVersion {
    git_tree: String,
    version: Option<String>,
    version_date: Option<String>,
    version_semver: Option<String>,
    version_string: Option<String>,
    port_version: u32,
}

// asc
#[derive(Clone, Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
pub struct VcpkgGitTreeIndex {
    #[serde(skip)]
    path: String,

    index: HashMap<String, VcpkgGitTreeInfo>,

    check_point: GitCommitInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VcpkgGitTreeInfo {
    pub port_name: String,
    pub commit_hash: String,
    pub commit_date_time: String,
}

impl VcpkgPortTreeVersion {
    pub fn format_version_text(&self) -> String {
        let mut s = String::new();
        if let Some(v) = &self.version {
            s = v.clone();
        } else if let Some(v) = &self.version_date {
            s = v.clone();
        } else if let Some(v) = &self.version_string {
            s = v.clone();
        } else if let Some(v) = &self.version_semver {
            s = v.clone();
        }

        if self.port_version == 0 {
            s
        } else {
            format!("{}#{}", s, self.port_version)
        }
    }
}

impl VcpkgManager {
    pub fn index(&mut self) -> bool {
        let commits = self.get_commits();
        let latest_commit = &commits[commits.len() - 1];

        self.build_git_tree_index(&commits);

        if !self.build_search_index(latest_commit) {
            return false;
        }

        return true;
    }

    fn get_vcpkg_root_dir() -> String {
        VcpkgArgs::load(&config::system_paths::ConfigPath::vcpkg_toml(), true)
            .unwrap()
            .directory
            .unwrap_or(config::system_paths::DataPath::vcpkg_clone_dir())
    }

    pub fn get_port_versions(port: &str) -> Vec<(String, String, String)> {
        let mut results = vec![];

        let versions_port_json_path = config::system_paths::DataPath::vcpkg_versions_port_json_path(
            &Self::get_vcpkg_root_dir(),
            port,
        );
        if let Some(versions) = VcpkgPortVersions::load(&versions_port_json_path, false) {
            if let Some(git_tree_index) = VcpkgGitTreeIndex::load(
                &config::system_paths::DataPath::vcpkg_tree_index_json(),
                false,
            ) {
                for v in versions.versions {
                    if let Some(info) = git_tree_index.index.get(&v.git_tree) {
                        results.push((
                            v.format_version_text(),
                            info.commit_hash.clone(),
                            info.commit_date_time.clone(),
                        ));
                    } else {
                        tracing::error!("{:#?}", v)
                    }
                }
            }
        }

        return results;
    }

    fn build_search_index(&mut self, latest_commit: &GitCommitInfo) -> bool {
        let versions_baseline_json_path =
            config::system_paths::DataPath::vcpkg_versions_baseline_json_path(
                &Self::get_vcpkg_root_dir(),
            );
        match VcpkgBaseline::load(&versions_baseline_json_path, false) {
            None => return false,
            Some(baseline_data) => {
                let mut search_index = VcpkgSearchIndex::load(
                    &config::system_paths::DataPath::vcpkg_search_index_json(),
                    true,
                )
                .unwrap();
                if latest_commit.hash <= search_index.check_point.hash {
                    return true;
                }

                for port_name in baseline_data.default.keys() {
                    search_index
                        .prefix_trie
                        .insert(&port_name, port_name.clone());
                    search_index
                        .postfix_trie
                        .insert(&util::str::reverse_string(port_name), port_name.clone());
                }
                search_index.baseline = baseline_data;
                search_index.check_point = latest_commit.clone();
                return search_index.dump(false);
            }
        }
    }

    fn build_git_tree_index(&self, commits: &Vec<GitCommitInfo>) -> VcpkgGitTreeIndex {
        let mut results = VcpkgGitTreeIndex::load(
            &config::system_paths::DataPath::vcpkg_tree_index_json(),
            true,
        )
        .unwrap();

        let mut next_index = 0;
        if let Some(index) = commits
            .iter()
            .position(|c| c.hash == results.check_point.hash)
        {
            next_index = index + 1;
        }

        for (index, c) in commits[next_index..].iter().enumerate() {
            if index <= next_index {
                continue;
            }

            let trees = self.get_git_trees(&c.hash);
            for (git_tree, port_name) in &trees {
                if !results.index.contains_key(git_tree) {
                    results.index.insert(
                        git_tree.clone(),
                        VcpkgGitTreeInfo {
                            port_name: port_name.clone(),
                            commit_hash: c.hash.clone(),
                            commit_date_time: c.date_time.clone(),
                        },
                    );
                }
            }

            if index % 200 == 0 || commits.len() < 1000 {
                results.check_point = c.clone();
                results.dump(false);
                tracing::info!("[{index}] #{}# {:#?}", results.index.len(), c.date_time);
            }
        }

        results.check_point = commits[commits.len() - 1].clone();
        results.dump(false);

        return results;
    }

    fn get_git_trees(&self, git_commit_hash: &str) -> Vec<(String, String)> {
        let mut results = vec![];

        let cwd = util::fs::get_cwd();
        util::fs::set_cwd(&Self::get_vcpkg_root_dir());
        let output = util::shell::run(
            "git",
            &vec![
                "ls-tree",
                "-d",
                "-r",
                "--full-tree",
                git_commit_hash,
                VCPKG_PORTS_DIR_NAME,
            ],
            true,
            false,
            true,
        )
        .unwrap();
        util::fs::set_cwd(&cwd);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.split("\n") {
            let s = line.trim();
            if !s.is_empty() {
                let right = s.split_once(" tree ").unwrap().1;
                let parts: Vec<&str> = right
                    .split(VCPKG_PORTS_DIR_NAME)
                    .map(|s| s.trim())
                    .collect();
                if parts.len() == 2 {
                    results.push((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }

        return results;
    }

    pub fn get_latest_commit() -> GitCommitInfo {
        let cwd = util::fs::get_cwd();
        util::fs::set_cwd(&Self::get_vcpkg_root_dir());
        let output = util::shell::run(
            "git",
            &vec![
                "log",
                "-n 1",
                "--date=iso",
                r#"--pretty=format:{"hash": "%H", "date_time": "%ad"}"#,
            ],
            true,
            false,
            false,
        )
        .unwrap();
        util::fs::set_cwd(&cwd);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.split("\n") {
            match serde_json::from_str(line) {
                Err(e) => {
                    tracing::error!(
                        call = "serde_json::from_str",
                        line = line,
                        error_tag = ErrorTag::JsonDeserializeError.as_ref(),
                        message = e.to_string()
                    );
                }
                Ok(info) => {
                    return info;
                }
            }
        }

        return GitCommitInfo::default();
    }

    fn get_commits(&mut self) -> Vec<GitCommitInfo> {
        let mut commits = vec![];

        let cwd = util::fs::get_cwd();

        util::fs::set_cwd(&Self::get_vcpkg_root_dir());
        let output = util::shell::run(
            "git",
            &vec![
                "log",
                "--reverse",
                "--date=iso",
                r#"--pretty=format:{"hash": "%H", "date_time": "%ad"}"#,
            ],
            true,
            false,
            false,
        )
        .unwrap();
        util::fs::set_cwd(&cwd);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.split("\n") {
            match serde_json::from_str(line) {
                Err(e) => {
                    tracing::error!(
                        call = "serde_json::from_str",
                        line = line,
                        error_tag = ErrorTag::JsonDeserializeError.as_ref(),
                        message = e.to_string()
                    );
                }
                Ok(info) => {
                    commits.push(info);
                }
            }
        }

        return commits;
    }
}
