pub struct Requirement {
    required_child: i32,
}

impl Requirement {
    pub fn new(required_child: i32) -> Self {
        Self { required_child }
    }
    pub fn is_satisfied(&self, init_requirement: i32, remain_requirement: i32) -> bool {
        remain_requirement == 0 || init_requirement - remain_requirement >= self.required_child
    }
}
