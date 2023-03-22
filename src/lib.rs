pub mod routes;
pub mod utils;

pub mod nebula {
    use crate::utils::{db, nebula::NebulaCourse};
    pub fn fetch_courses() -> Vec<NebulaCourse> {
        db::create_client()
            .database("combinedDB")
            .collection::<NebulaCourse>("courses")
            .find(None, None)
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }
}
