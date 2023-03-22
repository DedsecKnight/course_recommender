use crate::utils::graph::CourseGraph;
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
    vertex_count: usize,
    edge_count: usize,
}

#[rocket::post("/", rocket::data = "<payload>")]
pub fn index(course_graph: &State<CourseGraph>, payload: Json<PrereqData>) -> Json<PrereqResponse> {
    println!("{:#?}", &payload.courses);
    let sz = course_graph.size();
    let edge_sz = course_graph.edges().len();
    Json(PrereqResponse {
        vertex_count: sz,
        edge_count: edge_sz,
    })
}
