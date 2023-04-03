use crate::{
    nebula::validate_degree,
    utils::{graph::CourseGraph, semester::SemesterData},
};
use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    State,
};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct PrereqData {
    semester: Vec<SemesterData>,
    bypasses: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PrereqResponse {
    is_valid: bool,
    invalid_reason: String,
}

#[rocket::post("/", rocket::data = "<payload>")]
pub fn index(course_graph: &State<CourseGraph>, payload: Json<PrereqData>) -> Json<PrereqResponse> {
    let semester_data: Vec<SemesterData> = payload.semester.clone();
    let course_sets = semester_data.iter().map(|x| x.courses.clone()).collect();
    match validate_degree(&course_sets, &payload.bypasses, course_graph) {
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
