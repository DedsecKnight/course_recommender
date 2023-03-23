use crate::{nebula::validate_degree, utils::graph::CourseGraph};
use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    State,
};

#[derive(Deserialize, Debug)]
pub struct PrereqData {
    courses: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PrereqResponse {
    is_valid: bool,
    invalid_reason: String,
}

#[rocket::post("/", rocket::data = "<payload>")]
pub fn index(course_graph: &State<CourseGraph>, payload: Json<PrereqData>) -> Json<PrereqResponse> {
    match validate_degree(&payload.courses, course_graph) {
        Ok(_) => Json(PrereqResponse {
            is_valid: true,
            invalid_reason: String::from(""),
        }),
        Err(err) => Json(PrereqResponse {
            is_valid: false,
            invalid_reason: err,
        }),
    }
}
