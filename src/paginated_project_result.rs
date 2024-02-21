use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatedProjectResult {
    pub pagination: Pagination,
    pub result: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pagination {
    limit: i64,
    offset: i64,
    count: i64,
}

impl Display for Pagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "limit : {}", self.limit)?;
        writeln!(f, "offset : {}", self.offset)?;
        writeln!(f, "count : {}", self.count)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub created_at: String,
    pub plugin_id: String,
    pub name: String,
    pub namespace: ProjectNamespace,
    pub promoted_versions: Vec<PromotedVersion>,
    pub stats: ProjectStatsAll,
    pub category: String, //todo : Enum
    pub description: String,
    pub last_updated: String,
    pub visibility: String,
    pub user_actions: UserActions,
    pub settings: ProjectSettings,
    pub icon_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedVersionResult {
    pagination: Pagination,
    result: Vec<Version>,
}

impl Display for PaginatedVersionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.result
                .iter()
                .map(|f| f.to_string())
                .collect::<String>()
        )
        //writeln!(f, "Pagination : {}", self.pagination)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    created_at: String,
    name: String,
    dependencies: Vec<VersionDependency>,
    visibility: String,
    description: Option<String>,
    stats: VersionStatsAll,
    file_info: FileInfo,
    author: Option<String>,
    review_state: String,
    tags: Vec<VersionTag>,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========={}========", self.name)?;
        writeln!(f, "Author : {}", self.author.as_deref().unwrap_or_default())?;
        writeln!(f, "Created at : {}", self.created_at)?;
        writeln!(f, "Review State : {}", self.review_state)?;

        //writeln!(
        //    f,
        //    "{}",
        //    self.dependencies
        //        .iter()
        //        .map(|d| d.to_string())
        //        .collect::<String>()
        //)?;
        //writeln!(f, "{}", self.visibility)?;
        //writeln!(f, "{}", self.description.as_deref().unwrap_or_default())?;
        writeln!(f, "Downloads : {}", self.stats)?;

        writeln!(f, "{}", self.file_info)

        //writeln!(
        //    f,
        //    "{}",
        //    self.tags.iter().map(|t| t.to_string()).collect::<String>()
        //)
    }
}

#[derive(Serialize, Deserialize)]
pub struct VersionStatsAll {
    downloads: i64,
}

impl Display for VersionStatsAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.downloads)
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    size_bytes: i64,
    md_5_hash: Option<String>,
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#======File Info=======")?;
        writeln!(f, "# Name : {}", self.name)?;
        writeln!(f, "# Bytes : {}", self.size_bytes)?;
        writeln!(
            f,
            "# md_5 : {}",
            self.md_5_hash.as_deref().unwrap_or("Not Available")
        )?;
        writeln!(f, "#======================")
    }
}

#[derive(Serialize, Deserialize)]
pub struct VersionTag {
    name: String,
    data: Option<String>,
    color: VersionTagColor,
}

impl Display for VersionTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        writeln!(f, "{}", self.data.as_deref().unwrap_or_default())?;
        writeln!(f, "{}", self.color)
    }
}

#[derive(Serialize, Deserialize)]
pub struct VersionDependency {
    plugin_id: String,
    version: Option<String>,
}

impl Display for VersionDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.plugin_id)?;
        writeln!(f, "{}", self.version.as_deref().unwrap_or_default())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectNamespace {
    pub owner: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromotedVersion {
    pub version: String,
    pub tags: Vec<PromotedVersionTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromotedVersionTag {
    pub name: String,
    pub data: Option<String>,
    pub display_data: Option<String>,
    pub minecraft_version: Option<String>,
    pub color: VersionTagColor,
}

impl Display for PromotedVersionTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.display_data.as_ref().unwrap(),)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VersionTagColor {
    foreground: String,
    background: String,
}

impl Display for VersionTagColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.foreground)?;
        writeln!(f, "{}", self.background)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectStatsAll {
    views: i64,
    downloads: i64,
    recent_views: i64,
    recent_downloads: i64,
    stars: i64,
    watchers: i64,
}

impl Display for ProjectStatsAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Views : {}", self.views)?;
        writeln!(f, "Recent Views : {}", self.recent_views)?;
        writeln!(f, "Downloads : {}", self.downloads)?;
        writeln!(f, "Recent Downloads : {}", self.recent_downloads)?;
        writeln!(f, "Stars : {}", self.stars)?;
        writeln!(f, "Watchers : {}", self.watchers)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserActions {
    starred: bool,
    watching: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    homepage: Option<String>,
    issues: Option<String>,
    sources: Option<String>,
    license: ProjectLicense,
    forum_sync: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectLicense {
    name: Option<String>,
    url: Option<String>,
}
