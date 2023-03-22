use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum RequirementCollectionType {
    COLLECTION,
    COURSE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NebulaCourse {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    course_number: String,
    subject_prefix: String,
    prerequisites: RequirementCollection,
    corequisites: RequirementCollection,
    co_or_pre_requisites: RequirementCollection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequirementCollection {
    #[serde(alias = "type")]
    collection_type: String,
    options: Option<Vec<RequirementCollection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_reference: Option<ObjectId>,
    pub required: Option<i32>,
}

impl NebulaCourse {
    pub fn name(&self) -> String {
        format!("{} {}", &self.subject_prefix, &self.course_number)
    }

    pub fn prerequisites(&self) -> &RequirementCollection {
        &self.prerequisites
    }

    pub fn corequisites(&self) -> &RequirementCollection {
        &self.corequisites
    }

    pub fn co_or_pre_requisites(&self) -> &RequirementCollection {
        &self.co_or_pre_requisites
    }
}

impl RequirementCollection {
    pub fn requirement_type(&self) -> Option<RequirementCollectionType> {
        match self.collection_type.as_ref() {
            "collection" => Some(RequirementCollectionType::COLLECTION),
            "course" => Some(RequirementCollectionType::COURSE),
            _ => None,
        }
    }
    pub fn subrequirements(&self) -> &Option<Vec<RequirementCollection>> {
        &self.options
    }
}
