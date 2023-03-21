use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NebulaCourse {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub course_number: String,
    pub subject_prefix: String,
    pub prerequisites: RequirementCollection,
    pub corequisites: RequirementCollection,
    pub co_or_pre_requisites: RequirementCollection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequirementCollection {
    #[serde(alias = "type")]
    pub collection_type: String,
    pub options: Option<Vec<RequirementCollection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_reference: Option<ObjectId>,
    pub required: Option<i32>,
}
