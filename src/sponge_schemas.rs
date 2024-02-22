use chrono::{DateTime, Utc};
use clap::ValueEnum;
use human_bytes::human_bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    created_at: String,
    plugin_id: String,
    name: String,
    namespace: ProjectNamespace,
    promoted_versions: Vec<PromotedVersion>,
    stats: ProjectStatsAll,
    category: Category,
    description: String,
    last_updated: String,
    visibility: String,
    user_actions: UserActions,
    settings: ProjectSettings,
    icon_url: String,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Plugin : {}", self.name)?;
        writeln!(f, "Author : {}", self.namespace.owner)?;
        writeln!(f, "Description : {}", self.description)?;
        writeln!(
            f,
            "Last Updated : {}",
            self.last_updated.parse::<DateTime<Utc>>().unwrap()
        )?;
        writeln!(
            f,
            "Promoted Version : {}",
            self.promoted_versions
                .iter()
                .map(|f| format!(
                    "{} - {}",
                    f.version.clone(),
                    f.tags
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join("-")
                ))
                .collect::<Vec<String>>()
                .join("\n\t| ")
        )?;
        writeln!(f, "{}", self.stats)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Role {
    name: String,
    title: String,
    color: String,
}

#[derive(ValueEnum, Clone, Serialize, Deserialize, Debug)]
pub enum Category {
    admin_tools,
    chat,
    dev_tools,
    economy,
    gameplay,
    games,
    protection,
    role_playing,
    world_management,
    misc,
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Category::admin_tools => "Admin Tools",
            Category::chat => "Chat",
            Category::dev_tools => "Dev Tools",
            Category::economy => "Economy",
            Category::gameplay => "Gameplay",
            Category::games => "Games",
            Category::protection => "Protection",
            Category::role_playing => "Role Playing",
            Category::world_management => "World Management",
            Category::misc => "Misc",
        };
        write!(f, "{}", str)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectLicense {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedApiKey {
    key: String,
    perms: Vec<String>, //todo : Replace with enum
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PermissionCheck {
    r#type: String, //todo : Replace with enum
    result: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromotedVersion {
    pub version: String,
    pub tags: Vec<PromotedVersionTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompactProject {
    plugin_id: String,
    name: String,
    namespace: ProjectNamespace,
    promoted_versions: Vec<PromotedVersion>,
    stats: ProjectStatsAll,
    category: Category,
    visibility: String, //todo : Replace with enum
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPermissions {
    r#type: String,           //todo : Replace with enum
    permissions: Vec<String>, // Ditto ^
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserActions {
    starred: bool,
    watching: bool,
}

#[derive(ValueEnum, Clone, Serialize, Deserialize, Debug)]
enum NamedPermissions {
    ViewPublicInfo,
    EditOwnUserSettings,
    EditApiKeys,
    EditSubjectSettings,
    ManageSubjectMembers,
    IsSubjectOwner,
    CreateProject,
    EditPage,
    DeleteProject,
    CreateVersion,
    EditVersion,
    DeleteVersion,
    EditTags,
    CreateOrganization,
    PostAsOrganization,
    ModNotesAndFlags,
    SeeHidden,
    IsStaff,
    Reviewer,
    ViewHealth,
    ViewIp,
    ViewStats,
    ViewLogs,
    ManualValueChanges,
    HardDeleteProject,
    HardDeleteVersion,
    EditAllUserSettings,
}

pub struct ApiSessionProperties {
    expires_in: Option<i64>,
}

pub struct User {
    created_at: String,
    name: String,
    tagline: Option<String>,
    join_date: Option<String>,
    roles: Vec<Role>,
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
pub struct ProjectSettings {
    homepage: Option<String>,
    issues: Option<String>,
    sources: Option<String>,
    license: ProjectLicense,
    forum_sync: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct KeyToCreate {
    name: String,
    permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatedProjectResult {
    pub pagination: Pagination,
    pub result: Vec<Project>,
}

impl Display for PaginatedProjectResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //writeln!(f, "{}", self.pagination)?;
        self.result
            .iter()
            .map(|p| writeln!(f, "{}", p.plugin_id))
            .collect::<std::fmt::Result>()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectStatsDay {
    downloads: i64,
    view: i64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatedCompactProjectResult {
    pagination: Pagination,
    result: Vec<CompactProject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeployVersionInfo {
    create_forum_post: bool,
    description: String,
    //tags: todo!(),
    // This is typed in documentation as
    // < * > : { oneOf -> String
    //                    Vec<String>
    // }
    //
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

#[derive(Serialize, Deserialize)]
pub struct VersionTag {
    name: String,
    data: Option<String>,
    color: VersionTagColor,
}

impl Display for VersionTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        write!(f, ":{}", self.data.as_deref().unwrap_or_default())
        //writeln!(f, "{}", self.color)
    }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectNamespace {
    pub owner: String,
    pub slug: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnedApiSession {
    pub session: String,
    pub expires: String, //date-time
    pub r#type: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    size_bytes: f64,
    md_5_hash: Option<String>,
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#=====[File Info]======")?;
        writeln!(f, "# Name : {}", self.name)?;
        writeln!(f, "# Bytes : {}", human_bytes(self.size_bytes))?;
        writeln!(
            f,
            "# md_5 : {}",
            self.md_5_hash.as_deref().unwrap_or("Not Available")
        )?;
        writeln!(f, "#======================")
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct VersionStatsDay {
    downloads: i64,
}
#[derive(ValueEnum, Clone, Serialize, Deserialize, Debug)]
pub enum ProjectSortingStrategy {
    Stars,
    Downloads,
    Views,
    Newest,
    Updated,
    OnlyRelevance,
    RecentDownloads,
    RecentViews,
}

impl Display for ProjectSortingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ProjectSortingStrategy::Stars => "Stars",
            ProjectSortingStrategy::Downloads => "Downloads",
            ProjectSortingStrategy::Views => "Views",
            ProjectSortingStrategy::Newest => "Newest",
            ProjectSortingStrategy::Updated => "Updated",
            ProjectSortingStrategy::OnlyRelevance => "Only Relevance",
            ProjectSortingStrategy::RecentDownloads => "Recent Downloads",
            ProjectSortingStrategy::RecentViews => "Recent Views",
        };
        write!(f, "{}", str)
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
        writeln!(
            f,
            "Created at : {}",
            self.created_at.parse::<DateTime<Utc>>().unwrap()
        )?;
        writeln!(f, "Review State : {}", self.review_state)?;
        writeln!(
            f,
            "Tags : {}",
            self.tags
                .iter()
                .map(|t| format!("[{}] ", t))
                .collect::<String>()
        )?;
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
pub struct ProjectMember {
    user: String,
    roles: Vec<Role>,
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

#[derive(Serialize, Deserialize)]
pub struct VersionStatsAll {
    downloads: i64,
}

impl Display for VersionStatsAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.downloads)
    }
}
