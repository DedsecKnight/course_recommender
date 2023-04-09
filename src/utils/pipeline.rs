use std::collections::VecDeque;

use petgraph::{graph::NodeIndex, Direction};

use crate::utils::graph::{EdgeType, VertexType};

use super::graph::CourseGraph;

struct CourseRecommenderState {
    indegree: Vec<usize>,
    fulfilled: Vec<bool>,
}

impl CourseRecommenderState {
    fn new(g: &CourseGraph) -> Self {
        Self {
            indegree: g
                .graph()
                .node_indices()
                .map(|x| g.graph().neighbors_directed(x, Direction::Incoming).count())
                .collect(),
            fulfilled: vec![false; g.size()],
        }
    }
}

pub struct CourseRecommenderPipeline<'a> {
    state: CourseRecommenderState,
    g: &'a CourseGraph,
}

impl<'a> CourseRecommenderPipeline<'a> {
    pub fn new(g: &'a CourseGraph) -> Self {
        Self {
            state: CourseRecommenderState::new(g),
            g,
        }
    }
    pub fn process(
        &mut self,
        taken_courses: Vec<Vec<String>>,
        bypasses: Vec<String>,
    ) -> Result<(), String> {
        let course_sets = self.validate_and_remove_seminar_requirement(taken_courses)?;
        self.validate_degree(&course_sets, &bypasses)
    }
    fn validate_degree(
        &mut self,
        taken_courses: &Vec<Vec<String>>,
        bypasses: &Vec<String>,
    ) -> Result<(), String> {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
        enum NodeState {
            Prerequisite,
            Corequisite,
            Neither,
        }
        let mut q: VecDeque<(NodeIndex, NodeState)> = VecDeque::new();
        for course in bypasses {
            if let Some(course_index) = self.g.find_course_by_name(course) {
                self.state.indegree[course_index.index()] = 0;
                self.state.fulfilled[course_index.index()] = true;
                q.push_back((course_index, NodeState::Neither));
            } else {
                return Err(format!("Invalid course found: {}", course));
            }
        }

        while let Some((curr_node, curr_state)) = q.pop_front() {
            match curr_state {
                NodeState::Neither => {
                    assert_eq!(self.g.node_type(curr_node.index()), VertexType::COURSE);
                    let mut edges = self
                        .g
                        .graph()
                        .neighbors_directed(curr_node, Direction::Outgoing)
                        .detach();
                    while let Some((edge, neighbor)) = edges.next(self.g.graph()) {
                        if self.state.indegree[neighbor.index()] > 0 {
                            self.state.indegree[neighbor.index()] -= 1;
                        }
                        assert_eq!(self.g.node_type(neighbor.index()), VertexType::REQUIREMENT);
                        if !self.state.fulfilled[neighbor.index()]
                            && self.g.requirement_satisfied(
                                neighbor,
                                self.state.indegree[neighbor.index()],
                            )
                        {
                            if self.g.graph()[edge] == EdgeType::COREQUISITE {
                                q.push_back((neighbor, NodeState::Corequisite));
                            } else {
                                q.push_back((neighbor, NodeState::Prerequisite));
                            }
                            self.state.fulfilled[neighbor.index()] = true;
                            self.state.indegree[neighbor.index()] = 0;
                        }
                    }
                }
                other_state => {
                    assert_eq!(self.g.node_type(curr_node.index()), VertexType::REQUIREMENT);
                    for neighbor in self
                        .g
                        .graph()
                        .neighbors_directed(curr_node, Direction::Outgoing)
                        .by_ref()
                    {
                        if self.state.fulfilled[neighbor.index()] {
                            continue;
                        }
                        if self.state.indegree[neighbor.index()] > 0 {
                            self.state.indegree[neighbor.index()] -= 1;
                        }
                        if self.g.node_type(neighbor.index()) == VertexType::COURSE
                            && curr_state == NodeState::Corequisite
                            && self.g.course_group_satisfied(neighbor, bypasses)
                            && self.state.indegree[neighbor.index()] == 0
                        {
                            q.push_back((neighbor, NodeState::Neither));
                            self.state.fulfilled[neighbor.index()] = true;
                            self.state.indegree[neighbor.index()] = 0;
                        }
                        if self.g.node_type(neighbor.index()) == VertexType::REQUIREMENT
                            && self.g.requirement_satisfied(
                                neighbor,
                                self.state.indegree[neighbor.index()],
                            )
                        {
                            q.push_back((neighbor, other_state));
                            self.state.fulfilled[neighbor.index()] = true;
                            self.state.indegree[neighbor.index()] = 0;
                        }
                    }
                }
            }
        }

        for semester_set in taken_courses {
            for course in semester_set {
                if let Some(course_index) = self.g.find_course_by_name(course) {
                    if self.g.course_group_satisfied(course_index, semester_set)
                        && self.state.indegree[course_index.index()] == 0
                    {
                        self.state.fulfilled[course_index.index()] = true;
                        q.push_back((course_index, NodeState::Neither));
                    }
                } else {
                    return Err(format!("Invalid course found: {}", course));
                }
            }
            while !q.is_empty() {
                while let Some((curr_node, curr_state)) = q.pop_front() {
                    match curr_state {
                        NodeState::Neither => {
                            assert_eq!(self.g.node_type(curr_node.index()), VertexType::COURSE);
                            let mut edges = self
                                .g
                                .graph()
                                .neighbors_directed(curr_node, Direction::Outgoing)
                                .detach();
                            while let Some((edge, neighbor)) = edges.next(self.g.graph()) {
                                if self.state.indegree[neighbor.index()] > 0 {
                                    self.state.indegree[neighbor.index()] -= 1;
                                }
                                assert_eq!(
                                    self.g.node_type(neighbor.index()),
                                    VertexType::REQUIREMENT
                                );
                                if !self.state.fulfilled[neighbor.index()]
                                    && self.g.requirement_satisfied(
                                        neighbor,
                                        self.state.indegree[neighbor.index()],
                                    )
                                {
                                    if self.g.graph()[edge] == EdgeType::COREQUISITE {
                                        q.push_back((neighbor, NodeState::Corequisite));
                                    } else {
                                        q.push_back((neighbor, NodeState::Prerequisite));
                                    }
                                    self.state.fulfilled[neighbor.index()] = true;
                                    self.state.indegree[neighbor.index()] = 0;
                                }
                            }
                        }
                        other_state => {
                            assert_eq!(
                                self.g.node_type(curr_node.index()),
                                VertexType::REQUIREMENT
                            );
                            for neighbor in self
                                .g
                                .graph()
                                .neighbors_directed(curr_node, Direction::Outgoing)
                                .by_ref()
                            {
                                if self.state.fulfilled[neighbor.index()] {
                                    continue;
                                }
                                if self.state.indegree[neighbor.index()] > 0 {
                                    self.state.indegree[neighbor.index()] -= 1;
                                }
                                if self.g.node_type(neighbor.index()) == VertexType::COURSE
                                    && curr_state == NodeState::Corequisite
                                    && self.g.course_group_satisfied(neighbor, semester_set)
                                    && self.state.indegree[neighbor.index()] == 0
                                {
                                    q.push_back((neighbor, NodeState::Neither));
                                    self.state.fulfilled[neighbor.index()] = true;
                                    self.state.indegree[neighbor.index()] = 0;
                                }
                                if self.g.node_type(neighbor.index()) == VertexType::REQUIREMENT
                                    && self.g.requirement_satisfied(
                                        neighbor,
                                        self.state.indegree[neighbor.index()],
                                    )
                                {
                                    q.push_back((neighbor, other_state));
                                    self.state.fulfilled[neighbor.index()] = true;
                                    self.state.indegree[neighbor.index()] = 0;
                                }
                            }
                        }
                    }
                }
                for course in semester_set {
                    if let Some(course_index) = self.g.find_course_by_name(course) {
                        if !self.g.course_group_satisfied(course_index, semester_set) {
                            return Err(format!(
                                "Found course with unsatisfied group: {}",
                                &course
                            ));
                        }
                        if self.state.indegree[course_index.index()] > 0 {
                            return Err(format!(
                                "Found course with unfulfilled pre/corequisites: {} (Need {} more requirement(s))",
                                &course,
                                &self.state.indegree[course_index.index()]
                            ));
                        }
                        if !self.state.fulfilled[course_index.index()] {
                            q.push_back((course_index, NodeState::Neither));
                            self.state.fulfilled[course_index.index()] = true;
                        }
                    }
                }
            }
        }

        Ok(())
    }
    fn validate_and_remove_seminar_requirement(
        &mut self,
        taken_courses: Vec<Vec<String>>,
    ) -> Result<Vec<Vec<String>>, String> {
        let mut ret: Vec<Vec<String>> = vec![];
        let valid_courses = [
            "ARHM 1100",
            "ATCM 1100",
            "BBSU 1100",
            "BCOM 1300",
            "BIS 1100",
            "ECS 1100",
            "EPPS 1110",
            "NATS 1101",
            "NATS 1142",
            "UNIV 1100",
        ];
        let can_pair_seminar =
            |course_name: &String| -> bool { valid_courses.iter().any(|x| *x == course_name) };
        for course_set in taken_courses {
            if course_set.iter().any(|x| x == "UNIV 1010")
                ^ course_set.iter().any(|x| can_pair_seminar(x))
            {
                return Err(String::from("Missing corequisite for seminar requirement"));
            } else {
                ret.push(
                    course_set
                        .into_iter()
                        .filter(|x| x != "UNIV 1010" && !valid_courses.iter().any(|y| *y == x))
                        .collect(),
                );
            }
        }
        Ok(ret)
    }
}
