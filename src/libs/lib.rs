pub mod query {
    use std::fmt::Display;

    /// Builds a set of arguments to build a query for a link
    /// Returns a [Vec]<([String],[String])>
    ///
    /// Takes a [str] and [QueryType]
    /// ```
    /// # use oremon_lib::query::{Query, QueryType};
    /// # use oremon_lib::{plugin_response, query_builder};
    /// #
    /// let query = query_builder!("q" : QueryType::Value(Some("value"))).to_vec();
    /// assert_eq!(query, vec![("q".to_string(),"value".to_string())]);
    /// ```
    /// ```
    /// # use oremon_lib::query::{Query, QueryType};
    /// # use oremon_lib::{plugin_response, query_builder};
    /// #
    /// let query_vec = query_builder!(
    ///     "list" : QueryType::Vec(Some(vec!["one","two","three"]))
    /// ).to_vec();
    ///
    /// let result : Vec<(String,String)> = vec![
    ///         ("list","one"),
    ///         ("list","two"),
    ///         ("list","three")
    ///     ]
    ///     .iter()
    ///     .map(|f| (f.0.to_string(),f.1.to_string()))
    ///     .collect();
    /// assert_eq!(query_vec, result);
    /// ```
    #[macro_export]
    macro_rules! query_builder {
        ($($lit:literal : $val:expr),+ $(,)?) => {
            {
                use std::collections::HashMap;
                use oremon_lib::query::{Query, QueryType};

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

    use crate::mc_mod_info::{McModInfo, ModInfo};

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
        /// ```
        /// # use oremon_lib::file_reader::FileReader;
        /// # use std::path::Path;
        /// # use oremon_lib::mc_mod_info::McModInfo;
        /// let reader = FileReader::from(Path::new("./local/test/"));
        /// let file = reader.handle_dir().unwrap();
        /// let mod_one = McModInfo {
        ///     modid : "nucleus".to_string(),
        ///     name : "Nucleus".to_string(),
        ///     version : "2.1.4".to_string(),
        /// };
        /// let mod_two = McModInfo {
        ///     modid : "huskycrates".to_string(),
        ///     name : "HuskyCrates".to_string(),
        ///     version : "2.0.0PRE9H2".to_string(),
        /// };
        /// let mods = vec![mod_one, mod_two];
        /// assert_eq!(file,mods);
        /// ```
        pub fn handle_dir(&self) -> Result<Vec<McModInfo>> {
            let info = fs::read_dir(&self.base_path)?
                .filter_map(|res| res.ok())
                .map(|entry| entry.path())
                .filter_map(|path| self.handle_file(Some(&path)).ok())
                .map(|f| f.info)
                .collect::<Vec<McModInfo>>();

            Ok(info)
        }

        /// The file intended to be read from
        const INFO_FILE: &'static str = "mcmod.info";

        /// Handles a single file. It reads from the [PathBuf] provided.
        /// If a path is provided it will read from it instead.
        /// ```
        /// # use oremon_lib::file_reader::FileReader;
        /// # use std::path::Path;
        /// # use oremon_lib::mc_mod_info::McModInfo;
        /// let reader = FileReader::from(Path::new("./local/test/nucleus.jar"));
        /// let file = reader.handle_file(None).unwrap();
        /// let mod_info = McModInfo {
        ///     modid : "nucleus".to_string(),
        ///     name : "Nucleus".to_string(),
        ///     version : "2.1.4".to_string(),
        /// };
        /// assert_eq!(file.info,mod_info);
        /// ```
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
}

/// Module handles version checking implementation
pub mod version_status {
    use std::fmt::Display;
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
        /// Compares two [Versioning]s
        /// The first parameter should be the remote version.
        /// The second parameter should be the local version.
        ///
        /// ```
        /// # use oremon_lib::version_status::VersionStatus;
        /// assert_eq!(VersionStatus::check_version("2.0","2.0"), VersionStatus::UpToDate);
        ///
        /// assert_eq!(VersionStatus::check_version("1.0","2.0"), VersionStatus::OutOfDate);
        ///
        /// assert_eq!(VersionStatus::check_version("2.0","1.0"), VersionStatus::Overdated);
        ///
        /// assert_ne!(VersionStatus::check_version("1.0","1.0"), VersionStatus::OutOfDate);
        /// ```
        pub fn check_version(local: &'_ str, remote: &'_ str) -> VersionStatus {
            let local = Versioning::new(&local).unwrap();
            let remote = Versioning::new(&remote).unwrap();
            if local == remote {
                VersionStatus::UpToDate
            } else if local < remote {
                VersionStatus::OutOfDate
            } else {
                VersionStatus::Overdated
            }
        }
    }
}

pub mod mc_mod_info {
    use serde::Deserialize;

    /// The root representation of an mcmod.info
    #[derive(Deserialize, Debug)]
    pub struct ModInfo {
        pub info: McModInfo,
    }

    /// A partial representation of a mcmod.info file
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct McModInfo {
        pub modid: String,
        pub name: String,
        pub version: String,
    }
}
