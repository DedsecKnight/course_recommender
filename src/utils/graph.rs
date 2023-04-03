use bson::oid::ObjectId;

use super::course::Course;
use super::nebula::{NebulaCourse, RequirementCollection, RequirementCollectionType};
use super::requirement::Requirement;
use std::collections::HashMap;

use petgraph::graph::{Graph, NodeIndex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeType {
    PREREQUISITE,
    COREQUISITE,
    SUBREQUIREMENT,
}

#[derive(PartialEq, Debug)]
pub enum VertexType {
    COURSE,
    REQUIREMENT,
}
pub struct CourseGraphBuilder {
    graph_size: usize,
    courses: HashMap<NodeIndex, Course>,
    requirements: HashMap<NodeIndex, Requirement>,
    graph: Graph<usize, EdgeType>,
    id_to_course_index: HashMap<ObjectId, NodeIndex>,
    course_name_to_index: HashMap<String, NodeIndex>,
    course_group: HashMap<String, Vec<NodeIndex>>,
}

pub struct CourseGraph {
    courses: HashMap<NodeIndex, Course>,
    requirements: HashMap<NodeIndex, Requirement>,
    course_name_to_index: HashMap<String, NodeIndex>,
    graph: Graph<usize, EdgeType>,
    course_group: HashMap<String, Vec<NodeIndex>>,
}

impl CourseGraphBuilder {
    fn new() -> Self {
        Self {
            graph_size: 0usize,
            courses: HashMap::new(),
            graph: Graph::new(),
            requirements: HashMap::new(),
            id_to_course_index: HashMap::new(),
            course_name_to_index: HashMap::new(),
            course_group: HashMap::new(),
        }
    }
    fn add_course(&mut self, course: &NebulaCourse) -> NodeIndex {
        let node_index = self.graph.add_node(self.graph_size);
        let course_group_key = course.course_key();
        self.courses.insert(
            node_index,
            Course::new(course.subject_prefix(), course.course_number()),
        );
        self.course_name_to_index.insert(course.name(), node_index);
        if !self.course_group.contains_key(&course_group_key) {
            self.course_group
                .insert(course_group_key.clone(), vec![node_index]);
        } else {
            self.course_group
                .get_mut(&course_group_key)
                .unwrap()
                .push(node_index);
        }
        self.id_to_course_index
            .insert(course.id.unwrap(), node_index);
        self.graph_size += 1;
        node_index
    }
    fn add_requirement(&mut self, required_child: u32) -> NodeIndex {
        let node_index = self.graph.add_node(self.graph_size);
        self.requirements
            .insert(node_index, Requirement::new(required_child));
        self.graph_size += 1;
        node_index
    }
    fn parse_requirement(
        &mut self,
        requirement: &RequirementCollection,
        root_course_group: &str,
    ) -> Option<NodeIndex> {
        if let Some(RequirementCollectionType::COLLECTION) = requirement.requirement_type() {
            let mut subrequirement_children: Vec<(NodeIndex, EdgeType)> = vec![];
            for subrequirement in requirement.subrequirements().iter().flatten() {
                if let Some(subreq_index) =
                    self.parse_requirement(subrequirement, root_course_group)
                {
                    subrequirement_children.push((subreq_index, EdgeType::SUBREQUIREMENT));
                }
            }
            if subrequirement_children.is_empty() {
                None
            } else {
                let requirement_node = self.add_requirement(std::cmp::min(
                    subrequirement_children.len(),
                    requirement.required.unwrap() as usize,
                ) as u32);
                for (subreq_index, requirement_type) in subrequirement_children {
                    self.graph
                        .add_edge(subreq_index, requirement_node, requirement_type);
                }
                Some(requirement_node)
            }
        } else if let Some(RequirementCollectionType::COURSE) = requirement.requirement_type() {
            if let Some(course_id) = requirement.class_reference {
                let course_index = self.id_to_course_index.get(&course_id).unwrap();
                if self
                    .course_group
                    .get(root_course_group)
                    .unwrap()
                    .contains(&course_index)
                {
                    None
                } else {
                    self.id_to_course_index.get(&course_id).copied()
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn build_from_db(course_database: &Vec<NebulaCourse>) -> CourseGraph {
        let mut g = Self::default();
        for course in course_database {
            g.add_course(course);
        }
        for course in course_database {
            let course_id = course.id.unwrap();
            let course_index = g.id_to_course_index.get(&course_id).unwrap().to_owned();
            let course_key = course.course_key();

            if let Some(prereq_index) = g.parse_requirement(course.prerequisites(), &course_key) {
                g.graph
                    .add_edge(prereq_index, course_index, EdgeType::PREREQUISITE);
            }

            if let Some(coreq_index) = g.parse_requirement(course.corequisites(), &course_key) {
                g.graph
                    .add_edge(coreq_index, course_index, EdgeType::COREQUISITE);
            }

            if let Some(coreq_index) =
                g.parse_requirement(course.co_or_pre_requisites(), &course_key)
            {
                g.graph
                    .add_edge(coreq_index, course_index, EdgeType::COREQUISITE);
            }
        }

        CourseGraph {
            courses: g.courses,
            requirements: g.requirements,
            course_name_to_index: g.course_name_to_index,
            graph: g.graph,
            course_group: g.course_group,
        }
    }
}

impl CourseGraph {
    pub fn find_course_by_name(&self, course_name: &str) -> Option<NodeIndex> {
        self.course_name_to_index.get(course_name).copied()
    }
    pub fn graph(&self) -> &Graph<usize, EdgeType> {
        &self.graph
    }
    pub fn size(&self) -> usize {
        self.graph.node_count()
    }
    pub fn node_type(&self, node_index: usize) -> VertexType {
        let node_key: NodeIndex<u32> = NodeIndex::new(node_index);
        if self.courses.contains_key(&node_key) {
            VertexType::COURSE
        } else {
            VertexType::REQUIREMENT
        }
    }
    pub fn course_group_satisfied(&self, node_index: NodeIndex, course_set: &Vec<String>) -> bool {
        let node_key = self.courses.get(&node_index).unwrap().course_key();
        for node in self.course_group.get(&node_key).unwrap() {
            let target_course_name = self.courses.get(node).unwrap().name();
            if !node.eq(&node_index) && course_set.contains(&target_course_name) {
                return true;
            }
        }
        return self.course_group.get(&node_key).unwrap().len() == 1;
    }
    pub fn requirement_satisfied(&self, node_index: NodeIndex, remain_degree: usize) -> bool {
        let init_degree = self
            .graph
            .neighbors_directed(node_index, petgraph::Direction::Incoming)
            .count();
        self.requirements
            .get(&node_index)
            .unwrap()
            .is_satisfied(init_degree as u32, remain_degree as u32)
    }
}

impl Default for CourseGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
