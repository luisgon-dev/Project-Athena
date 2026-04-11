use book_router::domain::{
    catalog::{WorkRecord, ResolvedWork, WorkSearch},
    requests::{
        CreateRequest, CreateRequestSelection, ManifestationPreference, MediaType,
        RequestDetailRecord, RequestListRecord, RequestRecord,
    },
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
    CreateRequestSelection::export_all_to("frontend/src/lib/types/").unwrap();
    RequestRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RequestListRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RequestDetailRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventKind::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventRecord::export_all_to("frontend/src/lib/types/").unwrap();
}
