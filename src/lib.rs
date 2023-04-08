pub mod routes;
pub mod utils;

pub mod nebula {
    use std::collections::VecDeque;

    use petgraph::{graph::NodeIndex, Direction};

    use crate::utils::{
        db,
        graph::{CourseGraph, EdgeType, VertexType},
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
        taken_courses: &Vec<Vec<String>>,
        bypasses: &Vec<String>,
        g: &CourseGraph,
    ) -> Result<(), String> {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
        enum NodeState {
            Prerequisite,
            Corequisite,
            Neither,
        }
        let mut q: VecDeque<(NodeIndex, NodeState)> = VecDeque::new();
        let mut indegree: Vec<usize> = g
            .graph()
            .node_indices()
            .map(|x| g.graph().neighbors_directed(x, Direction::Incoming).count())
            .collect();
        let mut fulfilled: Vec<bool> = vec![false; g.size()];

        for course in bypasses {
            if let Some(course_index) = g.find_course_by_name(course) {
                indegree[course_index.index()] = 0;
                fulfilled[course_index.index()] = true;
                q.push_back((course_index, NodeState::Neither));
            } else {
                return Err(format!("Invalid course found: {}", course));
            }
        }

        while let Some((curr_node, curr_state)) = q.pop_front() {
            match curr_state {
                NodeState::Neither => {
                    assert_eq!(g.node_type(curr_node.index()), VertexType::COURSE);
                    let mut edges = g
                        .graph()
                        .neighbors_directed(curr_node, Direction::Outgoing)
                        .detach();
                    while let Some((edge, neighbor)) = edges.next(g.graph()) {
                        if indegree[neighbor.index()] > 0 {
                            indegree[neighbor.index()] -= 1;
                        }
                        assert_eq!(g.node_type(neighbor.index()), VertexType::REQUIREMENT);
                        if !fulfilled[neighbor.index()]
                            && g.requirement_satisfied(neighbor, indegree[neighbor.index()])
                        {
                            if g.graph()[edge] == EdgeType::COREQUISITE {
                                q.push_back((neighbor, NodeState::Corequisite));
                            } else {
                                q.push_back((neighbor, NodeState::Prerequisite));
                            }
                            fulfilled[neighbor.index()] = true;
                            indegree[neighbor.index()] = 0;
                        }
                    }
                }
                other_state => {
                    assert_eq!(g.node_type(curr_node.index()), VertexType::REQUIREMENT);
                    for neighbor in g
                        .graph()
                        .neighbors_directed(curr_node, Direction::Outgoing)
                        .by_ref()
                    {
                        if fulfilled[neighbor.index()] {
                            continue;
                        }
                        if indegree[neighbor.index()] > 0 {
                            indegree[neighbor.index()] -= 1;
                        }
                        if g.node_type(neighbor.index()) == VertexType::COURSE
                            && curr_state == NodeState::Corequisite
                            && g.course_group_satisfied(neighbor, bypasses)
                            && indegree[neighbor.index()] == 0
                        {
                            q.push_back((neighbor, NodeState::Neither));
                            fulfilled[neighbor.index()] = true;
                        } else if g.node_type(neighbor.index()) == VertexType::REQUIREMENT
                            && g.requirement_satisfied(neighbor, indegree[neighbor.index()])
                        {
                            q.push_back((neighbor, other_state));
                            fulfilled[neighbor.index()] = true;
                            indegree[neighbor.index()] = 0;
                        }
                    }
                }
            }
        }

        for semester_set in taken_courses {
            for course in semester_set {
                if let Some(course_index) = g.find_course_by_name(course) {
                    if g.course_group_satisfied(course_index, semester_set)
                        && indegree[course_index.index()] == 0
                    {
                        fulfilled[course_index.index()] = true;
                        q.push_back((course_index, NodeState::Neither));
                    }
                } else {
                    return Err(format!("Invalid course found: {}", course));
                }
            }
            while let Some((curr_node, curr_state)) = q.pop_front() {
                match curr_state {
                    NodeState::Neither => {
                        assert_eq!(g.node_type(curr_node.index()), VertexType::COURSE);
                        let mut edges = g
                            .graph()
                            .neighbors_directed(curr_node, Direction::Outgoing)
                            .detach();
                        while let Some((edge, neighbor)) = edges.next(g.graph()) {
                            if indegree[neighbor.index()] > 0 {
                                indegree[neighbor.index()] -= 1;
                            }
                            assert_eq!(g.node_type(neighbor.index()), VertexType::REQUIREMENT);
                            if !fulfilled[neighbor.index()]
                                && g.requirement_satisfied(neighbor, indegree[neighbor.index()])
                            {
                                if g.graph()[edge] == EdgeType::COREQUISITE {
                                    q.push_back((neighbor, NodeState::Corequisite));
                                } else {
                                    q.push_back((neighbor, NodeState::Prerequisite));
                                }
                                fulfilled[neighbor.index()] = true;
                                indegree[neighbor.index()] = 0;
                            }
                        }
                    }
                    other_state => {
                        assert_eq!(g.node_type(curr_node.index()), VertexType::REQUIREMENT);
                        for neighbor in g
                            .graph()
                            .neighbors_directed(curr_node, Direction::Outgoing)
                            .by_ref()
                        {
                            if fulfilled[neighbor.index()] {
                                continue;
                            }
                            if indegree[neighbor.index()] > 0 {
                                indegree[neighbor.index()] -= 1;
                            }
                            if g.node_type(neighbor.index()) == VertexType::COURSE
                                && curr_state == NodeState::Corequisite
                                && g.course_group_satisfied(neighbor, semester_set)
                                && indegree[neighbor.index()] == 0
                            {
                                q.push_back((neighbor, NodeState::Neither));
                                fulfilled[neighbor.index()] = true;
                            } else if g.node_type(neighbor.index()) == VertexType::REQUIREMENT
                                && g.requirement_satisfied(neighbor, indegree[neighbor.index()])
                            {
                                q.push_back((neighbor, other_state));
                                fulfilled[neighbor.index()] = true;
                                indegree[neighbor.index()] = 0;
                            }
                        }
                    }
                }
            }
            for course in semester_set {
                if let Some(course_index) = g.find_course_by_name(course) {
                    if !g.course_group_satisfied(course_index, semester_set)
                        || indegree[course_index.index()] > 0
                    {
                        return Err(format!("Found course with unsatisfied group: {}", &course));
                    }
                    if indegree[course_index.index()] > 0 {
                        return Err(format!(
                            "Found course with unfulfilled pre/: {} (Need {} more requirement(s))",
                            &course,
                            &indegree[course_index.index()]
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
