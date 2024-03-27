pub mod query {
    use std::fmt::Display;

    /// Builds a set of arguments to build a query for a link
    /// Returns a [Vec]<([String],[String])>
    ///
    /// Takes a [str] and [QueryType]
    /// ```
    /// use ore_monitor::query::{Query, QueryType};
    /// use ore_monitor::{plugin_response, query_builder};
    ///
    /// let query = query_builder!("q" : QueryType::Value(Some("value"))).to_vec();
    /// assert_eq!(query, vec![("q".to_string(),"value".to_string())]);
    /// ```
    /// ```
    /// use ore_monitor::query::{Query, QueryType};
    /// use ore_monitor::{plugin_response, query_builder};
    ///
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
    ///     .map(|(list,num)| (list.to_string(), num.to_string()))
    ///     .collect();
    /// assert_eq!(query_vec, result);
    /// ```
    #[macro_export]
    macro_rules! query_builder {
        ($($lit:literal : $val:expr),+ $(,)?) => {
            {
                use std::collections::HashMap;
                use ore_monitor::query::{Query, QueryType};

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

    /// Meant for Single value tuple Enums that share a common trait
    /// self is expected to be an enum
    /// path should be each of the Variants of the enum
    ///
    /// ```
    /// use ore_monitor::gen_matches;
    /// trait CommonTrait {}
    ///
    /// struct A {}
    /// impl CommonTrait for A {}
    ///
    /// struct B {}
    /// impl CommonTrait for B {}
    ///
    /// enum Enum {
    ///     Foo(A),
    ///     Bar(B),
    /// }
    ///
    /// impl Enum {
    ///     fn trait_value(&self) -> &dyn CommonTrait {
    ///         gen_matches!(self, Enum::Foo, Enum::Bar)
    ///     }
    /// }
    /// ```
    /// The macro expands into
    /// ```
    /// # use ore_monitor::gen_matches;
    /// # trait CommonTrait {}
    /// #
    /// # struct A {}
    /// # impl CommonTrait for A {}
    /// #
    /// # struct B {}
    /// # impl CommonTrait for B {}
    /// #
    /// # enum Enum {
    /// #     Foo(A),
    /// #     Bar(B),
    /// # }
    /// impl Enum {
    ///     fn trait_value(&self) -> &dyn CommonTrait {
    ///         match self {
    ///             Enum::Foo(value) => value,
    ///             Enum::Bar(value) => value,
    ///         }
    ///     }
    /// }
    ///
    /// ```
    #[macro_export]
    macro_rules! gen_matches {
        ($self:ident, $($path:path),*) => {
            match $self {
                $($path(value) => value,)+
            }
        };
    }

    #[macro_export]
    macro_rules! plugin_response {
        ($plugin_id:expr,$ore_client:expr) => {{
            let link = format!("/projects/{}", $plugin_id);
            $ore_client.get(link, None)
        }};
    }

    /// Query represents a list of arguments found in a URL as Key/Values
    pub struct Query {
        pub query: Vec<(String, String)>,
    }

    impl Query {
        pub fn new(query: Vec<(String, String)>) -> Self {
            Query { query }
        }

        pub fn get_query(&self, key: &str) -> String {
            self.query
                .iter()
                .filter(|(k, _)| k == key)
                .map(|(_, f)| f.to_string())
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
        io::{BufReader, Read},
        ops::Deref,
        path::{Path, PathBuf},
    };

    use anyhow::Result;

    use serde::de::DeserializeOwned;
    use zip::ZipArchive;

    use crate::ore_mod_info::{ModInfo, OreModInfo, PluginInfo};

    /// A reader that takes a [PathBuf] to read a file or group of files
    #[derive(Debug, Default)]
    pub struct FileReader {
        pub base_path: PathBuf,
    }

    enum FileTypes {
        InfoFile,
        PluginFile,
    }

    impl FileTypes {
        pub fn try_get(&self, jar_reader: &mut JarFileReader) -> Result<OreModInfo> {
            match self {
                FileTypes::InfoFile => jar_reader
                    .find_file::<ModInfo>("mcmod.info")
                    .map(Into::into),
                FileTypes::PluginFile => jar_reader
                    .find_file::<PluginInfo>("META-INF/sponge_plugins.json")
                    .map(Into::into),
            }
        }
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
        /// # use ore_monitor::file_reader::FileReader;
        /// # use std::path::Path;
        /// # use ore_monitor::ore_mod_info::McModInfo;
        /// # use ore_monitor::ore_mod_info::OreModInfo;
        /// let reader = FileReader::from(Path::new("./local/test/"));
        /// let file = reader.handle_dir().unwrap();
        /// let mod_one : OreModInfo = McModInfo {
        ///     modid : "nucleus".to_string(),
        ///     name : "Nucleus".to_string(),
        ///     version : "2.1.4".to_string(),
        ///     dependencies : vec!["spongeapi@7.3".to_string()],
        ///     required_mods : vec!["spongeapi@7.3".to_string()]
        /// }.into();
        /// let mod_two: OreModInfo  = McModInfo {
        ///     modid : "huskycrates".to_string(),
        ///     name : "HuskyCrates".to_string(),
        ///     version : "2.0.0PRE9H2".to_string(),
        ///     dependencies : vec![
        ///         "placeholderapi".to_string(),
        ///         "spongeapi@7.1.0-SNAPSHOT".to_string(),
        ///         "huskyui@0.6.0PRE3".to_string()
        ///     ],
        ///     required_mods : vec![
        ///         "spongeapi@7.1.0-SNAPSHOT".to_string(),
        ///         "huskyui@0.6.0PRE3".to_string()
        ///     ]
        /// }.into();
        /// let mods = vec![mod_one, mod_two];
        /// assert_eq!(file, mods);
        /// ```
        pub fn handle_dir(&self) -> Result<Vec<OreModInfo>> {
            let info = fs::read_dir(&self.base_path)?
                .filter_map(|res| res.ok())
                .map(|entry| entry.path())
                .filter_map(|path| self.handle_file(Some(&path)).ok())
                .collect::<Vec<OreModInfo>>();

            Ok(info)
        }

        /// Handles a single file. It reads from the [PathBuf] provided.
        /// If a path is provided it will read from it instead.
        /// ```
        /// # use ore_monitor::file_reader::FileReader;
        /// # use ore_monitor::ore_mod_info::McModInfo;
        /// # use ore_monitor::ore_mod_info::OreModInfo;
        /// # use std::path::Path;
        /// let reader = FileReader::from(Path::new("./local/test/nucleus.jar"));
        /// let file = reader.handle_file(None).unwrap();
        /// let mod_info : OreModInfo = McModInfo {
        ///     modid : "nucleus".to_string(),
        ///     name : "Nucleus".to_string(),
        ///     version : "2.1.4".to_string(),
        ///     dependencies : vec!["spongeapi@7.3".to_string()],
        ///     required_mods : vec!["spongeapi@7.3".to_string()]
        /// }.into();
        /// assert_eq!(file,mod_info);
        /// ```
        pub fn handle_file(&self, path: Option<&Path>) -> Result<OreModInfo> {
            Ok(path.unwrap_or(self.base_path.deref()))
                .and_then(|path| File::open(path))
                .and_then(|file| Ok(BufReader::new(file)))
                .and_then(|buf_reader| Ok(ZipArchive::new(buf_reader)))?
                .and_then(|zip| Ok(JarFileReader::new(zip)))
                .and_then(|mut jar_reader| {
                    Ok(FileTypes::InfoFile
                        .try_get(&mut jar_reader)
                        .or_else(|_e| FileTypes::PluginFile.try_get(&mut jar_reader)))
                })?
        }
    }

    /// JarFileReader is intended to read `.jar` files
    struct JarFileReader {
        file: ZipArchive<BufReader<File>>,
    }

    impl JarFileReader {
        fn new(file: ZipArchive<BufReader<File>>) -> Self {
            JarFileReader { file }
        }

        /// Locates a file from a [ZipArchive] by the files name
        /// Returns [self] for method chaining.
        fn find_file<T: DeserializeOwned>(&mut self, file_name: &str) -> Result<T> {
            let mut buf = String::new();
            self.file.by_name(file_name)?.read_to_string(&mut buf)?;
            Ok(serde_json::from_str::<T>(&buf)?)
        }
    }
}

pub mod ore_mod_info {
    use serde::Deserialize;

    /// A generic representation of both McMod.info and sponge_plugins.json
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct OreModInfo {
        pub modid: String,
        pub name: String,
        pub version: String,
        pub major_api_version: u32,
    }

    impl OreModInfo {
        fn new(modid: String, name: String, version: String, major_api_version: u32) -> Self {
            OreModInfo {
                modid,
                name,
                version,
                major_api_version,
            }
        }
    }

    impl From<ModInfo> for OreModInfo {
        fn from(value: ModInfo) -> Self {
            value.info.into()
        }
    }

    impl From<McModInfo> for OreModInfo {
        fn from(value: McModInfo) -> Self {
            let info = value;
            let major = info.sponge_tag_version();
            OreModInfo::new(info.modid, info.name, info.version, major)
        }
    }

    impl From<PluginInfo> for OreModInfo {
        fn from(value: PluginInfo) -> Self {
            let plugin = value.first_plugin();
            OreModInfo::new(
                plugin.id,
                plugin.name,
                plugin.version.unwrap_or_default().replace(' ', "-"),
                value.major_api_version(),
            )
        }
    }

    /// The root representation of an mcmod.info
    #[derive(Deserialize, Debug)]
    pub struct ModInfo {
        pub info: McModInfo,
    }

    /// A partial representation of a mcmod.info file
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct McModInfo {
        pub modid: String,
        pub name: String,
        pub version: String,
        pub dependencies: Vec<String>,
        pub required_mods: Vec<String>,
    }

    impl McModInfo {
        /// Attempts to get the tag version from the mcmod file
        /// First reading from the dependencies list, if failing that the required_mods list.
        /// ```
        /// use ore_monitor::ore_mod_info::McModInfo;
        ///
        /// let mod_info = McModInfo {
        ///     modid : "nucleus".to_string(),
        ///     name : "Nucleus".to_string(),
        ///     version : "2.1.4".to_string(),
        ///     dependencies : vec!["spongeapi@7.3".to_string()],
        ///     required_mods : vec!["spongeapi@7.3".to_string()]
        /// };
        ///
        /// let tag = mod_info.sponge_tag_version();
        /// assert_eq!(tag, 7);
        /// ```
        pub fn sponge_tag_version(&self) -> u32 {
            self.find_major_version("spongeapi", &self.dependencies)
                .or(self.find_major_version("spongeapi", &self.required_mods))
                .unwrap_or_default()
        }

        fn find_major_version(&self, id: &'_ str, list: &Vec<String>) -> Option<u32> {
            list.iter()
                .find(|str| str.starts_with(id))
                .and_then(|str| str.split_once('@'))
                .and_then(|(_, ver)| ver.split_once('.'))
                .and_then(|(major, _)| major.parse().ok())
        }
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct PluginInfo {
        pub global: Option<GlobalPlugin>,
        pub plugins: Vec<PluginData>,
    }

    impl PluginInfo {
        fn first_plugin(&self) -> PluginData {
            self.plugins
                .first()
                .unwrap_or(&PluginData::default())
                .clone()
        }

        fn major_api_version(&self) -> u32 {
            self.global
                .clone()
                .and_then(|f| Some(f.dependencies))
                .or(Some(self.first_plugin().dependencies))
                .unwrap_or_default()
                .iter()
                .filter(|dep| dep.is_sponge_dep())
                .map(|ver| ver.major_api_version())
                .collect::<Vec<u32>>()
                .first()
                .and_then(|f| Some(f.clone()))
                .unwrap_or_default()
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Clone)]
    pub struct GlobalPlugin {
        pub version: String,
        pub dependencies: Vec<PluginDependency>,
    }

    #[derive(Debug, Deserialize, PartialEq, Default, Clone)]
    pub struct PluginDependency {
        pub id: String,
        pub version: String,
    }

    impl PluginDependency {
        fn is_sponge_dep(&self) -> bool {
            self.id.eq_ignore_ascii_case("spongeapi")
        }

        fn major_api_version(&self) -> u32 {
            self.version
                .split_once('.')
                .and_then(|(major, _)| major.parse().ok())
                .unwrap_or_default()
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Clone, Default)]
    pub struct PluginData {
        pub id: String,
        pub name: String,
        pub version: Option<String>,
        pub dependencies: Vec<PluginDependency>,
    }
}
