pub mod routes;
pub mod utils;

pub mod nebula {
    use std::collections::{HashSet, VecDeque};

    use crate::utils::{
        db,
        graph::{CourseGraph, EdgeType},
        nebula::NebulaCourse,
    };
    pub fn fetch_courses() -> Vec<NebulaCourse> {
        db::create_client()
            .database("combinedDB")
            .collection::<NebulaCourse>("courses")
            .find(None, None)
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
    }
    pub fn validate_degree(
        taken_courses: &Vec<String>,
        course_graph: &CourseGraph,
    ) -> Result<(), String> {
        let mut course_set: HashSet<usize> = HashSet::new();
        for course in taken_courses {
            if let Some(course_index) = course_graph.find_course_by_name(course) {
                course_set.insert(course_index);
            } else {
                return Err(format!("Invalid course found: {}", course));
            }
        }

        let mut prereq_adj: Vec<Vec<usize>> = vec![vec![]; course_graph.size()];
        let mut coreq_adj: Vec<Vec<usize>> = vec![vec![]; course_graph.size()];

        let mut prereq_init_degree: Vec<i32> = vec![0; course_graph.size()];
        let mut coreq_init_degree: Vec<i32> = vec![0; course_graph.size()];

        let mut prereq_degree: Vec<i32> = vec![0; course_graph.size()];
        let mut coreq_degree: Vec<i32> = vec![0; course_graph.size()];

        for (u, v, edge_type) in course_graph.edges() {
            match edge_type {
                EdgeType::PREREQUISITE => {
                    prereq_adj[v.to_owned()].push(*u);
                    prereq_init_degree[*u] += 1;
                    prereq_degree[*u] += 1;
                }
                EdgeType::COREQUISITE => {
                    coreq_adj[v.to_owned()].push(*u);
                    coreq_init_degree[*u] += 1;
                    coreq_degree[*u] += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = VecDeque::new();

        for i in 0usize..course_graph.size() {
            if !course_graph.is_course_node(i) && prereq_degree[i] == 0 && coreq_degree[i] == 0 {
                queue.push_back(i);
            }
        }

        while !queue.is_empty() {
            let current_node = queue.pop_front().unwrap();
            for prereq_pred in &prereq_adj[current_node] {
                prereq_degree[prereq_pred.to_owned()] -= 1;
                // do something else here
                if !course_graph.is_course_node(*prereq_pred)
                    && course_graph
                        .get_requirement(*prereq_pred)
                        .unwrap()
                        .is_satisfied(
                            prereq_init_degree[*prereq_pred],
                            prereq_degree[*prereq_pred],
                        )
                {
                    queue.push_back(prereq_pred.to_owned());
                }
            }

            for coreq_pred in &coreq_adj[current_node] {
                coreq_degree[coreq_pred.to_owned()] -= 1;
                // do something else here
                if !course_graph.is_course_node(*coreq_pred)
                    && course_graph
                        .get_requirement(*coreq_pred)
                        .unwrap()
                        .is_satisfied(coreq_init_degree[*coreq_pred], coreq_degree[*coreq_pred])
                {
                    queue.push_back(coreq_pred.to_owned());
                }
            }
        }

        for i in 0usize..course_graph.size() {
            if course_set.contains(&i) && prereq_degree[i] == 0 && coreq_degree[i] == 0 {
                queue.push_back(i);
                course_set.remove(&i);
            }
        }
        while !queue.is_empty() {
            let current_node = queue.pop_front().unwrap();
            for prereq_pred in &prereq_adj[current_node] {
                prereq_degree[prereq_pred.to_owned()] -= 1;
                // do something else here
                if course_graph.is_course_node(*prereq_pred)
                    && prereq_degree[prereq_pred.to_owned()] == 0
                    && coreq_degree[prereq_pred.to_owned()] == 0
                {
                    if course_set.contains(prereq_pred) {
                        queue.push_back(prereq_pred.to_owned());
                        course_set.remove(prereq_pred);
                    }
                } else if !course_graph.is_course_node(*prereq_pred)
                    && course_graph
                        .get_requirement(*prereq_pred)
                        .unwrap()
                        .is_satisfied(
                            prereq_init_degree[*prereq_pred],
                            prereq_degree[*prereq_pred],
                        )
                {
                    queue.push_back(prereq_pred.to_owned());
                }
            }

            for coreq_pred in &coreq_adj[current_node] {
                coreq_degree[coreq_pred.to_owned()] -= 1;
                // do something else here
                if course_graph.is_course_node(*coreq_pred)
                    && prereq_degree[coreq_pred.to_owned()] == 0
                    && coreq_degree[coreq_pred.to_owned()] == 0
                {
                    if course_set.contains(coreq_pred) {
                        queue.push_back(coreq_pred.to_owned());
                        course_set.remove(coreq_pred);
                    }
                } else if !course_graph.is_course_node(*coreq_pred)
                    && course_graph
                        .get_requirement(*coreq_pred)
                        .unwrap()
                        .is_satisfied(prereq_init_degree[*coreq_pred], prereq_degree[*coreq_pred])
                {
                    queue.push_back(coreq_pred.to_owned());
                }
            }
        }

        if !course_set.is_empty() {
            let mut invalid_courses = String::from("");
            for course in course_set.iter() {
                invalid_courses.push_str(course_graph.get_course(*course).unwrap().name());
                invalid_courses.push_str(", ");
                println!(
                    "{} {} {}",
                    course_graph.get_course(*course).unwrap().name(),
                    prereq_degree[*course],
                    coreq_degree[*course]
                );
            }
            invalid_courses.pop();
            invalid_courses.pop();
            Err(format!(
                "Did not fulfill pre/corequisites for the following courses: {}",
                invalid_courses
            ))
        } else {
            Ok(())
        }
    }
}
