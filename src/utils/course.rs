#[derive(Debug)]
pub struct Course {
    subject_prefix: String,
    course_number: String,
}

impl Course {
    pub fn new(subject_prefix: &str, course_number: &str) -> Self {
        Self {
            subject_prefix: String::from(subject_prefix),
            course_number: String::from(course_number),
        }
    }
    pub fn name(&self) -> String {
        format!("{} {}", &self.subject_prefix, &self.course_number)
    }
    pub fn course_key(&self) -> String {
        format!(
            "{}_{}{}",
            &self.subject_prefix,
            &self.course_number[0..1],
            &self.course_number[self.course_number.len() - 2..]
        )
    }
}
