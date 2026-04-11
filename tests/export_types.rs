use book_router::domain::{
    catalog::{WorkRecord, ResolvedWork, WorkSearch},
    requests::{MediaType, ManifestationPreference, CreateRequest, RequestRecord, RequestListRecord},
    events::{RequestEventKind, RequestEventRecord},
};
use ts_rs::TS;

#[test]
fn export_types() {
    WorkRecord::export_all_to("frontend/src/lib/types/").unwrap();
    ResolvedWork::export_all_to("frontend/src/lib/types/").unwrap();
    WorkSearch::export_all_to("frontend/src/lib/types/").unwrap();
    MediaType::export_all_to("frontend/src/lib/types/").unwrap();
    ManifestationPreference::export_all_to("frontend/src/lib/types/").unwrap();
    CreateRequest::export_all_to("frontend/src/lib/types/").unwrap();
    RequestRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RequestListRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventKind::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventRecord::export_all_to("frontend/src/lib/types/").unwrap();
}
