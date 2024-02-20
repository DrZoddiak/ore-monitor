use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PaginatedProjectResult {
    pub pagination: Pagination,
    pub result: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Pagination {
    limit: i64,
    offset: i64,
    count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Project {
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ProjectNamespace {
    pub owner: String,
    slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PromotedVersion {
    pub version: String,
    pub tags: Vec<PromotedVersionTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PromotedVersionTag {
    pub name: String,
    pub data: Option<String>,
    pub display_data: Option<String>,
    pub minecraft_version: Option<String>,
    pub color: VersionTagColor,
}

impl Display for PromotedVersionTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.name,
            self.display_data.as_ref().unwrap(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct VersionTagColor {
    foreground: String,
    background: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ProjectStatsAll {
    views: i64,
    downloads: i64,
    recent_views: i64,
    recent_downloads: i64,
    stars: i64,
    watchers: i64,
}

impl Display for ProjectStatsAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n\t| Views: {}\n\t| Downloads: {}\n\t| Recent Views: {}\n\t| Recent Downloads: {}\n\t| Stars: {}\n\t| Watchers: {}",
            self.views,
            self.downloads,
            self.recent_views,
            self.recent_downloads,
            self.stars,
            self.watchers
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UserActions {
    starred: bool,
    watching: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ProjectSettings {
    homepage: Option<String>,
    issues: Option<String>,
    sources: Option<String>,
    license: ProjectLicense,
    forum_sync: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ProjectLicense {
    name: Option<String>,
    url: Option<String>,
}
