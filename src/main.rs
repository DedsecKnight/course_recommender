#[macro_use]
extern crate rocket;

use course_recommender::nebula;
use course_recommender::routes::validator;
use course_recommender::utils::graph::CourseGraphBuilder;
use dotenv::dotenv;

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let courses = nebula::fetch_courses();
    let course_graph = CourseGraphBuilder::build_from_db(&courses);
    rocket::build()
        .manage(course_graph)
        .mount("/validate", routes![validator::index])
}
