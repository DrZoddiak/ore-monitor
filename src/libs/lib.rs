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
            use common::QueryType;
            use common::Query;

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
