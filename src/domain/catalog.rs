#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkRecord {
    pub external_id: String,
    pub title: String,
    pub primary_author: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedWork {
    pub work: WorkRecord,
}
