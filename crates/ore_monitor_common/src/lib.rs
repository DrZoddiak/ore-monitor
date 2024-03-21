/// Module handles version checking implementation
pub mod version_status {
    use std::{cmp::Ordering, fmt::Display};
    use versions::Versioning;

    /// Represents the status a version can have compared to Ore
    #[derive(PartialEq, Debug)]
    pub enum VersionStatus {
        /// Version is outdated
        OutOfDate,
        /// Version is up-to-date
        UpToDate,
        /// Version is higher than remote version
        Overdated,
    }

    impl Display for VersionStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let status = match self {
                VersionStatus::OutOfDate => "Version is outdated",
                VersionStatus::UpToDate => "Version is up to date",
                VersionStatus::Overdated => "Local version is newer than Remote version",
            };
            write!(f, "{}", status)
        }
    }

    impl VersionStatus {
        /// Compares the local and remote versions
        /// ```
        /// use ore_monitor_common::version_status::VersionStatus;
        ///
        /// assert_eq!(VersionStatus::new("2.0","2.0"), VersionStatus::UpToDate);
        ///
        /// assert_eq!(VersionStatus::new("1.0","2.0"), VersionStatus::OutOfDate);
        ///
        /// assert_eq!(VersionStatus::new("2.0","1.0"), VersionStatus::Overdated);
        /// ```
        pub fn new(local: &'_ str, remote: &'_ str) -> VersionStatus {
            let local = Versioning::new(local).unwrap_or_default();
            let remote = Versioning::new(remote).unwrap_or_default();

            match local.cmp(&remote) {
                Ordering::Less => VersionStatus::OutOfDate,
                Ordering::Equal => VersionStatus::UpToDate,
                Ordering::Greater => VersionStatus::Overdated,
            }
        }
    }
}
