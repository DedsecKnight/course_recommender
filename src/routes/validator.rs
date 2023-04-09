use crate::utils::{
    graph::CourseGraph, pipeline::CourseRecommenderPipeline, semester::SemesterData,
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
    let semester_data: Vec<SemesterData> = payload.semester.to_owned();
    match CourseRecommenderPipeline::new(&course_graph).process(
        semester_data.into_iter().map(|x| x.courses).collect(),
        payload.bypasses.to_owned(),
    ) {
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
