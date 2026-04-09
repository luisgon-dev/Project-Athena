use super::requests::MediaType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
}
