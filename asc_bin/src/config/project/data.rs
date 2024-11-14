use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use config_file_derives::ConfigFile;
use config_file_types;

#[derive(Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub description: String,
    pub license: String,
    pub repository: String,
}

#[derive(Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
pub struct EntryConfig {
    pub name: String,
    pub source_dir: String,
    pub source_file: String,
    pub shared: Option<bool>,
}

#[derive(Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
pub struct DependencyConfig {
    pub version: String,
    pub find_packages: BTreeSet<String>,
    pub link_libraries: BTreeSet<String>,
    pub features: BTreeSet<String>,
}

#[derive(Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
pub struct WorkSpaceConfig {
    pub members: BTreeSet<String>,
}

#[derive(
    Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize, ConfigFile,
)]
#[config_file_ext("toml")]
pub struct ProjectConfig {
    pub workspace: Option<WorkSpaceConfig>,
    pub package: Option<PackageConfig>,
    #[serde(rename = "bin", default, skip_serializing_if = "BTreeSet::is_empty")]
    pub bins: BTreeSet<EntryConfig>,
    #[serde(rename = "lib", default, skip_serializing_if = "BTreeSet::is_empty")]
    pub libs: BTreeSet<EntryConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<String, BTreeSet<String>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, DependencyConfig>,

    #[serde(skip)]
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("toml")]
pub struct InstalledFiles {
    pub prefix: String,
    pub files: Vec<String>,

    #[serde(skip)]
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
pub struct VcpkgDependency {
    pub dependencies: Vec<VcpkgDependencyDesc>,
    pub overrides: Vec<BTreeMap<String, String>>,

    #[serde(skip)]
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct VcpkgDependencyDesc {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<BTreeMap<String, String>>,
}

#[derive(Debug, Default, Deserialize, Serialize, ConfigFile)]
#[config_file_ext("json")]
#[serde(rename_all = "kebab-case")]
pub struct VcpkgConfiguration {
    pub default_registry: VcpkgDefaultRegistry,
    pub registries: Vec<VcpkgRegistry>,

    #[serde(skip)]
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct VcpkgRegistry {
    pub kind: String,
    pub name: String,
    pub location: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct VcpkgDefaultRegistry {
    pub kind: String,
    pub baseline: String,
    pub repository: String,
}
