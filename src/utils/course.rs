#[derive(Debug)]
pub struct Course {
    course_name: String,
}

impl Course {
    pub fn new(name: String) -> Self {
        Self { course_name: name }
    }
    pub fn name(&self) -> &str {
        &self.course_name
    }
}
