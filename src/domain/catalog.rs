#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkRecord {
    pub external_id: String,
    pub title: String,
    pub primary_author: String,
    pub first_publish_year: Option<i32>,
    pub description: Option<String>,
    pub cover_id: Option<i64>,
    pub subjects: Vec<String>,
    pub edition_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedWork {
    pub work: WorkRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkSearch {
    pub works: Vec<WorkRecord>,
}
