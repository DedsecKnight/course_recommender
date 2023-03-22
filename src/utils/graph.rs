use bson::oid::ObjectId;

use super::course::Course;
use super::nebula::{NebulaCourse, RequirementCollection, RequirementCollectionType};
use super::requirement::Requirement;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum EdgeType {
    PREREQUISITE,
    COREQUISITE,
}

pub struct CourseGraph {
    graph_size: usize,
    courses: HashMap<usize, Course>,
    requirements: HashMap<usize, Requirement>,
    edges: Vec<(usize, usize, EdgeType)>,
    id_to_course_index: HashMap<ObjectId, usize>,
}

impl CourseGraph {
    pub fn size(&self) -> usize {
        self.graph_size
    }
    pub fn edges(&self) -> &Vec<(usize, usize, EdgeType)> {
        &self.edges
    }
    pub fn get_course(&self, course_id: usize) -> Option<&Course> {
        self.courses.get(&course_id)
    }
    pub fn get_requirement(&self, course_id: usize) -> Option<&Requirement> {
        self.requirements.get(&course_id)
    }
    pub fn new() -> Self {
        Self {
            graph_size: 0,
            courses: HashMap::new(),
            edges: vec![],
            requirements: HashMap::new(),
            id_to_course_index: HashMap::new(),
        }
    }
    fn add_course(&mut self, course: &NebulaCourse) -> usize {
        self.courses
            .insert(self.graph_size, Course::new(course.name()));
        self.id_to_course_index
            .insert(course.id.unwrap(), self.graph_size);
        self.graph_size += 1;
        self.graph_size - 1
    }
    fn add_requirement(&mut self, requirement: &RequirementCollection) -> usize {
        self.requirements.insert(
            self.graph_size,
            Requirement::new(requirement.required.unwrap()),
        );
        self.graph_size += 1;
        self.graph_size - 1
    }
    fn add_edge(&mut self, source_vertex: usize, dest_vertex: usize, edge_type: EdgeType) {
        self.edges.push((source_vertex, dest_vertex, edge_type));
    }

    fn parse_requirement(
        &mut self,
        requirement: &RequirementCollection,
        requirement_type: EdgeType,
    ) -> Option<usize> {
        if let Some(RequirementCollectionType::COLLECTION) = requirement.requirement_type() {
            let requirement_node = self.add_requirement(requirement);
            for subrequirement in requirement.subrequirements().iter().flatten() {
                if let Some(subreq_index) = self.parse_requirement(subrequirement, requirement_type)
                {
                    self.add_edge(requirement_node, subreq_index, requirement_type);
                }
            }
            Some(requirement_node)
        } else if let Some(RequirementCollectionType::COURSE) = requirement.requirement_type() {
            if let Some(course_index) = requirement.class_reference {
                self.id_to_course_index.get(&course_index).copied()
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn build_from_db(course_database: &Vec<NebulaCourse>) -> Self {
        let mut graph = Self::default();
        for course in course_database {
            graph.add_course(course);
        }
        for course in course_database {
            let course_id = course.id.unwrap();
            let course_index = graph.id_to_course_index.get(&course_id).unwrap().to_owned();

            if let Some(prereq_index) =
                graph.parse_requirement(course.prerequisites(), EdgeType::PREREQUISITE)
            {
                graph.add_edge(course_index, prereq_index, EdgeType::PREREQUISITE);
            }

            if let Some(coreq_index) =
                graph.parse_requirement(course.corequisites(), EdgeType::COREQUISITE)
            {
                graph.add_edge(course_index, coreq_index, EdgeType::COREQUISITE);
            }

            if let Some(coreq_index) =
                graph.parse_requirement(course.co_or_pre_requisites(), EdgeType::COREQUISITE)
            {
                graph.add_edge(course_index, coreq_index, EdgeType::COREQUISITE);
            }
        }
        graph
    }
}

impl Default for CourseGraph {
    fn default() -> Self {
        Self::new()
    }
}
