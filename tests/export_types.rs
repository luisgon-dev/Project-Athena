use book_router::domain::{
    catalog::{ResolvedWork, WorkRecord, WorkSearch},
    events::{RequestEventKind, RequestEventRecord},
    library::{LibraryScanJobRecord, LibraryScanResponse},
    requests::{
        CreateRequest, CreateRequestSelection, ManifestationPreference, MediaType,
        RequestDetailRecord, RequestListRecord, RequestRecord,
    },
    search::{ReleaseCandidate, ReviewQueueEntry},
    settings::{
        AcquisitionSettingsRecord, AcquisitionSettingsUpdate, AudiobookshelfIntegrationRecord,
        AudiobookshelfIntegrationUpdate, ConnectionTestResult, DownloadClientSettingsRecord,
        DownloadClientSettingsUpdate, ImportSettingsRecord, ImportSettingsUpdate,
        IntegrationSettingsRecord, IntegrationSettingsUpdate, MetadataSettingsRecord,
        MetadataSettingsUpdate, ProwlarrIntegrationRecord, ProwlarrIntegrationUpdate,
        QbittorrentSettingsRecord, QbittorrentSettingsUpdate, RuntimeSettingsRecord,
        RuntimeSettingsUpdate, StorageSettingsRecord, StorageSettingsUpdate, SyncedIndexerRecord,
    },
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
    ReleaseCandidate::export_all_to("frontend/src/lib/types/").unwrap();
    ReviewQueueEntry::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventKind::export_all_to("frontend/src/lib/types/").unwrap();
    RequestEventRecord::export_all_to("frontend/src/lib/types/").unwrap();
    StorageSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    StorageSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    MetadataSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    MetadataSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    QbittorrentSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    QbittorrentSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    ProwlarrIntegrationRecord::export_all_to("frontend/src/lib/types/").unwrap();
    ProwlarrIntegrationUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    AudiobookshelfIntegrationRecord::export_all_to("frontend/src/lib/types/").unwrap();
    AudiobookshelfIntegrationUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    ImportSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    ImportSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    AcquisitionSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    AcquisitionSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    DownloadClientSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    DownloadClientSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    IntegrationSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    IntegrationSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    RuntimeSettingsRecord::export_all_to("frontend/src/lib/types/").unwrap();
    RuntimeSettingsUpdate::export_all_to("frontend/src/lib/types/").unwrap();
    SyncedIndexerRecord::export_all_to("frontend/src/lib/types/").unwrap();
    ConnectionTestResult::export_all_to("frontend/src/lib/types/").unwrap();
    LibraryScanJobRecord::export_all_to("frontend/src/lib/types/").unwrap();
    LibraryScanResponse::export_all_to("frontend/src/lib/types/").unwrap();
}
