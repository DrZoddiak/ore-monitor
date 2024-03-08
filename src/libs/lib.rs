pub mod query {
    use std::fmt::Display;

    /// Builds a set of arguments to build a query for a link
    /// Returns a [Vec]<([String],[String])>
    ///
    /// Takes a [str] and [QueryType]
    /// ```
    /// use oremonlib::query;
    ///
    /// let query = query! {
    ///     "q" : QueryType::Value(Some("value"))
    /// }.to_vec();
    /// assert_eq!(query, vec![("q".to_string(),"value".to_string())]);
    /// ```
    /// ```
    /// use oremonlib::query;
    ///
    /// let query_vec = query! {
    ///     "list" : QueryType::Vec(Some(vec!["one","two","three"]))
    /// }.to_vec();
    ///
    /// let result : Vec<(String,String)> = vec![("list","one"),("list","two"),("list","three")].iter().map(|f| (f.0.to_string(),f.1.to_string())).collect();
    /// assert_eq!(query_vec, result);
    /// ```
    #[macro_export]
    macro_rules! query {
    ($($lit:literal : $val:expr),+ $(,)?) => {
        {
            use std::collections::HashMap;
            use crate::commands::query::Query;

            let mut map: HashMap<String, Vec<String>> = Default::default();

            $(
                if let Some(args) = $val.into() {
                    map.insert($lit.to_string(), args)
                } else {
                    None
                };
            )+

            let query = map.iter().map( |k| {
                k.1.iter().map(|v| (k.0.to_string(), v.to_string()))
            }).flatten().collect::<Vec<(String,String)>>();
            Query::new(query)
        }
    }
}

    #[macro_export]
    macro_rules! plugin_response {
        ($plugin_id:expr,$ore_client:expr) => {{
            let link = format!("/projects/{}", $plugin_id);
            $ore_client.get(link, None).await?
        }};
    }

    /// Query represents a list of arguments found in a URL as Key/Values
    pub struct Query {
        query: Vec<(String, String)>,
    }

    impl Query {
        pub fn new(query: Vec<(String, String)>) -> Self {
            Query { query }
        }

        pub fn get_query(&self, key: &str) -> String {
            self.query
                .iter()
                .filter(|k| k.0 == key)
                .map(|f| f.1.to_string())
                .collect::<String>()
        }

        pub fn to_vec(&self) -> Vec<(String, String)> {
            self.query.to_vec()
        }
    }

    /// Differentiates the difference between a Vec and Non-Vec value
    /// For the purposes of providing a clean [Display] impl
    pub enum QueryType<T: Display> {
        Vec(Option<Vec<T>>),
        Value(Option<T>),
    }

    impl<T: Display> Into<Option<Vec<String>>> for QueryType<T> {
        fn into(self) -> Option<Vec<String>> {
            match self {
                QueryType::Value(Some(e)) => Some(vec![e.to_string().to_lowercase()]),
                QueryType::Vec(Some(e)) => Some(e.iter().map(|f| f.to_string()).collect()),
                _ => None,
            }
        }
    }
}

pub mod file_reader {
    use std::{
        fs::{self, File},
        io::{BufReader, ErrorKind, Read, Result},
        ops::Deref,
        path::{Path, PathBuf},
    };

    use serde::de::DeserializeOwned;
    use zip::{read::ZipFile, ZipArchive};

    use crate::mc_mod_info::ModInfo;

    /// A reader that takes a [PathBuf] to read a file or group of files
    #[derive(Debug, Default)]
    pub struct FileReader {
        pub base_path: PathBuf,
    }

    impl FileReader {
        pub fn from(base_path: &Path) -> FileReader {
            Self {
                base_path: base_path.to_path_buf(),
            }
        }

        /// Handles a directory and reads the files inside of it
        /// Returns a Vector of [ModInfo] of each valid file.
        pub fn handle_dir(&self) -> Result<Vec<ModInfo>> {
            let info = fs::read_dir(&self.base_path)?
                .filter_map(|res| res.ok())
                .map(|entry| entry.path())
                .filter_map(|path| self.handle_file(Some(&path)).ok())
                .collect::<Vec<ModInfo>>();

            Ok(info)
        }

        /// The file intended to be read from
        const INFO_FILE: &'static str = "mcmod.info";

        /// Handles a single file. It reads from the [PathBuf] provided.
        /// If a path is provided it will read from it instead.
        pub fn handle_file(&self, path: Option<&Path>) -> Result<ModInfo> {
            let path = path.unwrap_or(self.base_path.deref());

            let file = File::open(path)?;

            let reader = BufReader::new(file);

            let mut zip = ZipArchive::new(reader)?;

            let jar_reader = JarFileReader::find_file(&mut zip, Self::INFO_FILE)?
                .read_file_content()?
                .deserialize::<ModInfo>()?;

            Ok(jar_reader)
        }
    }

    /// JarFileReader is intended to read `.jar` files
    struct JarFileReader<'a> {
        file: ZipFile<'a>,
        content: String,
    }

    impl<'a> JarFileReader<'a> {
        /// Locates a file from a [ZipArchive] by the files name
        /// Returns [self] for method chaining.
        fn find_file(
            zip: &'a mut ZipArchive<BufReader<File>>,
            str: &str,
        ) -> Result<JarFileReader<'a>> {
            Ok(Self {
                file: zip.by_name(str)?,
                content: String::new(),
            })
        }

        /// Reads the files content into a String which is then stored into [content]
        /// Returns [self] for method chaining.
        pub fn read_file_content(&mut self) -> Result<&mut Self> {
            self.file.read_to_string(&mut self.content)?;
            Ok(self)
        }

        /// Deserializes the returned content for Json files
        pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T> {
            serde_json::from_str(&self.content)
                .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
        }
    }

    #[cfg(test)]
    mod file_reader_tests {
        use std::path::Path;

        use crate::file_reader::FileReader;

        #[test]
        fn test_file_handle() {
            let reader = FileReader::from(Path::new("./local/nucleus.jar"));
            let info = reader.handle_file(None).unwrap();
            println!("{:?}", info)
        }

        #[test]
        fn test_dir_handle() {
            let reader = FileReader::from(Path::new("./local/"));
            let info = reader.handle_dir().unwrap();
            println!("{:?}", info)
        }
    }
}

pub mod version_status {
    use std::fmt::Display;
    use versions::Versioning;

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
            match self {
                VersionStatus::OutOfDate => writeln!(f, "Version is outdated"),
                VersionStatus::UpToDate => writeln!(f, "Version is up to date"),
                VersionStatus::Overdated => {
                    writeln!(f, "Source version is newer than Remote version")
                }
            }
        }
    }

    impl VersionStatus {
        pub fn check_version(remote: Versioning, source: Versioning) -> VersionStatus {
            if source == remote {
                VersionStatus::UpToDate
            } else if source < remote {
                VersionStatus::OutOfDate
            } else {
                VersionStatus::Overdated
            }
        }
    }

    #[cfg(test)]
    mod version_status_tests {
        use crate::mc_mod_info::McModInfo;
        use crate::version_status::VersionStatus;

        macro_rules! build_version {
            ($src:expr, $out:expr) => {{
                let v1 = McModInfo {
                    modid: "plugin".to_string(),
                    name: "Plugin".to_string(),
                    version: $src.to_string(),
                };
                let v2 = McModInfo {
                    modid: "plugin".to_string(),
                    name: "Plugin".to_string(),
                    version: $out.to_string(),
                };

                v1.check_version(v2)
            }};
        }

        #[test]
        fn matching_versions() {
            assert_eq!(build_version!("1.0", "1.0"), VersionStatus::UpToDate);
        }

        #[test]
        fn source_out_of_date() {
            assert_eq!(build_version!("1.0", "2.0"), VersionStatus::OutOfDate)
        }

        #[test]
        fn source_overdated() {
            assert_eq!(build_version!("2.0", "1.0"), VersionStatus::Overdated)
        }

        #[test]
        fn matching_versions_fail() {
            assert_ne!(build_version!("1.0", "1.0"), VersionStatus::OutOfDate);
        }

        #[test]
        fn source_out_of_date_fail() {
            assert_ne!(build_version!("1.0", "2.0"), VersionStatus::Overdated)
        }

        #[test]
        fn source_overdated_fail() {
            assert_ne!(build_version!("2.0", "1.0"), VersionStatus::UpToDate)
        }
    }
}

pub mod mc_mod_info {
    use serde::Deserialize;
    use versions::Versioning;

    use crate::version_status::VersionStatus;

    /// The root representation of an mcmod.info
    #[derive(Deserialize, Debug)]
    pub struct ModInfo {
        pub info: McModInfo,
    }

    /// A partial representation of a mcmod.info file
    #[derive(Deserialize, Debug)]
    pub struct McModInfo {
        pub modid: String,
        pub name: String,
        pub version: String,
    }

    impl McModInfo {
        pub fn check_version(&self, version: McModInfo) -> VersionStatus {
            let ore_ver = Versioning::new(&version.version).unwrap();
            let source_ver = Versioning::new(&self.version).unwrap();

            VersionStatus::check_version(ore_ver, source_ver)
        }
    }
}
