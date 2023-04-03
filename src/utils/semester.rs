use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq, PartialOrd, Clone)]
pub struct SemesterData {
    pub courses: Vec<String>,
}
