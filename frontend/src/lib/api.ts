import type { AuthBootstrapStatus } from '$lib/types/AuthBootstrapStatus';
import type { AuthUserRecord } from '$lib/types/AuthUserRecord';
import type { CreateSubmissionRequest } from '$lib/types/CreateSubmissionRequest';
import type { CreateUserRequest } from '$lib/types/CreateUserRequest';
import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
import type { AcquisitionSettingsRecord } from '$lib/types/AcquisitionSettingsRecord';
import type { AcquisitionSettingsUpdate } from '$lib/types/AcquisitionSettingsUpdate';
import type { AudiobookshelfIntegrationRecord } from '$lib/types/AudiobookshelfIntegrationRecord';
import type { AudiobookshelfIntegrationUpdate } from '$lib/types/AudiobookshelfIntegrationUpdate';
import type { ConnectionTestResult } from '$lib/types/ConnectionTestResult';
import type { LibraryScanJobRecord } from '$lib/types/LibraryScanJobRecord';
import type { LibraryScanResponse } from '$lib/types/LibraryScanResponse';
import type { ImportSettingsRecord } from '$lib/types/ImportSettingsRecord';
import type { ImportSettingsUpdate } from '$lib/types/ImportSettingsUpdate';
import type { LoginRequest } from '$lib/types/LoginRequest';
import type { NotificationSettingsRecord } from '$lib/types/NotificationSettingsRecord';
import type { NotificationSettingsUpdate } from '$lib/types/NotificationSettingsUpdate';
import type { ProwlarrIntegrationRecord } from '$lib/types/ProwlarrIntegrationRecord';
import type { ProwlarrIntegrationUpdate } from '$lib/types/ProwlarrIntegrationUpdate';
import type { QbittorrentSettingsRecord } from '$lib/types/QbittorrentSettingsRecord';
import type { QbittorrentSettingsUpdate } from '$lib/types/QbittorrentSettingsUpdate';
import type { RequestSubmissionDetailRecord } from '$lib/types/RequestSubmissionDetailRecord';
import type { RequestSubmissionRecord } from '$lib/types/RequestSubmissionRecord';
import type { ResolveManualSubmissionRequest } from '$lib/types/ResolveManualSubmissionRequest';
import type { RequestDetailRecord } from '$lib/types/RequestDetailRecord';
import type { RequestListRecord } from '$lib/types/RequestListRecord';
import type { RequestRecord } from '$lib/types/RequestRecord';
import type { RuntimeSettingsRecord } from '$lib/types/RuntimeSettingsRecord';
import type { RuntimeSettingsUpdate } from '$lib/types/RuntimeSettingsUpdate';
import type { SetupRequest } from '$lib/types/SetupRequest';
import type { StorageSettingsRecord } from '$lib/types/StorageSettingsRecord';
import type { StorageSettingsUpdate } from '$lib/types/StorageSettingsUpdate';
import type { SubmissionSearchResult } from '$lib/types/SubmissionSearchResult';
import type { SyncedIndexerRecord } from '$lib/types/SyncedIndexerRecord';
import type { UpdateUserRequest } from '$lib/types/UpdateUserRequest';
import type { UserRecord } from '$lib/types/UserRecord';
import type { WorkSearch } from '$lib/types/WorkSearch';

const API_PREFIX = '/api/v1';

type SearchParams = {
	title?: string;
	author?: string;
};

async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
	const response = await fetch(`${API_PREFIX}${path}`, init);

	if (!response.ok) {
		let message = `Request failed (${response.status})`;
		try {
			const payload = (await response.json()) as { error?: string };
			if (payload.error) {
				message = payload.error;
			}
		} catch {
			// Fall back to the default message when the backend does not send JSON.
		}
		throw new Error(message);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	return (await response.json()) as T;
}

function jsonRequest(path: string, method: string, body?: unknown): RequestInit {
	return {
		method,
		headers: { 'content-type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body)
	};
}

export function listRequests(): Promise<RequestListRecord[]> {
	return requestJson<RequestListRecord[]>('/requests');
}

export function getAuthBootstrap(): Promise<AuthBootstrapStatus> {
	return requestJson<AuthBootstrapStatus>('/auth/bootstrap');
}

export function setupAuth(payload: SetupRequest): Promise<AuthUserRecord> {
	return requestJson<AuthUserRecord>('/auth/setup', jsonRequest('auth-setup', 'POST', payload));
}

export function login(payload: LoginRequest): Promise<AuthUserRecord> {
	return requestJson<AuthUserRecord>('/auth/login', jsonRequest('auth-login', 'POST', payload));
}

export function logout(): Promise<void> {
	return requestJson<void>('/auth/logout', { method: 'POST' });
}

export function getCurrentUser(): Promise<AuthUserRecord> {
	return requestJson<AuthUserRecord>('/auth/me');
}

export function listUsers(): Promise<UserRecord[]> {
	return requestJson<UserRecord[]>('/users');
}

export function createUser(payload: CreateUserRequest): Promise<UserRecord> {
	return requestJson<UserRecord>('/users', jsonRequest('users-create', 'POST', payload));
}

export function updateUser(userId: string, payload: UpdateUserRequest): Promise<UserRecord> {
	return requestJson<UserRecord>(`/users/${userId}`, jsonRequest('users-update', 'PUT', payload));
}

export function searchRequests(params: SearchParams): Promise<WorkSearch> {
	const title = params.title?.trim() ?? '';
	const author = params.author?.trim() ?? '';
	if (!title && !author) {
		return Promise.resolve({ works: [] });
	}

	const query = new URLSearchParams();
	if (title) {
		query.set('title', title);
	}
	if (author) {
		query.set('author', author);
	}

	return requestJson<WorkSearch>(`/requests/search?${query.toString()}`);
}

export function createRequests(payload: CreateRequestSelection): Promise<RequestRecord[]> {
	return requestJson<RequestRecord[]>('/requests', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload)
	});
}

export function searchSubmissionCandidates(params: SearchParams): Promise<SubmissionSearchResult> {
	const title = params.title?.trim() ?? '';
	const author = params.author?.trim() ?? '';
	if (!title && !author) {
		return Promise.resolve({ works: [] });
	}

	const query = new URLSearchParams();
	if (title) query.set('title', title);
	if (author) query.set('author', author);
	return requestJson<SubmissionSearchResult>(`/submissions/search?${query.toString()}`);
}

export function listSubmissions(all = false): Promise<RequestSubmissionRecord[]> {
	return requestJson<RequestSubmissionRecord[]>(`/submissions${all ? '?all=true' : ''}`);
}

export function createSubmission(
	payload: CreateSubmissionRequest
): Promise<RequestSubmissionDetailRecord> {
	return requestJson<RequestSubmissionDetailRecord>(
		'/submissions',
		jsonRequest('submissions-create', 'POST', payload)
	);
}

export function getSubmissionDetail(submissionId: string): Promise<RequestSubmissionDetailRecord> {
	return requestJson<RequestSubmissionDetailRecord>(`/submissions/${submissionId}`);
}

export function resolveSubmission(
	submissionId: string,
	payload: ResolveManualSubmissionRequest
): Promise<RequestSubmissionDetailRecord> {
	return requestJson<RequestSubmissionDetailRecord>(
		`/submissions/${submissionId}/resolve`,
		jsonRequest('submissions-resolve', 'POST', payload)
	);
}

export function approveSubmission(submissionId: string): Promise<RequestSubmissionDetailRecord> {
	return requestJson<RequestSubmissionDetailRecord>(`/submissions/${submissionId}/approve`, {
		method: 'POST'
	});
}

export function rejectSubmission(submissionId: string): Promise<RequestSubmissionDetailRecord> {
	return requestJson<RequestSubmissionDetailRecord>(`/submissions/${submissionId}/reject`, {
		method: 'POST'
	});
}

export function getRequestDetail(requestId: string): Promise<RequestDetailRecord> {
	return requestJson<RequestDetailRecord>(`/requests/${requestId}`);
}

export function retryRequestSearch(requestId: string): Promise<RequestDetailRecord> {
	return requestJson<RequestDetailRecord>(`/requests/${requestId}/retry-search`, {
		method: 'POST'
	});
}

export function approveReviewCandidate(
	requestId: string,
	candidateId: bigint | number
): Promise<RequestDetailRecord> {
	return requestJson<RequestDetailRecord>(`/requests/${requestId}/review-queue/${candidateId}/approve`, {
		method: 'POST'
	});
}

export function rejectReviewCandidate(
	requestId: string,
	candidateId: bigint | number
): Promise<RequestDetailRecord> {
	return requestJson<RequestDetailRecord>(`/requests/${requestId}/review-queue/${candidateId}/reject`, {
		method: 'POST'
	});
}

export function getRuntimeSettings(): Promise<RuntimeSettingsRecord> {
	return requestJson<RuntimeSettingsRecord>('/settings/runtime');
}

export function updateRuntimeSettings(
	payload: RuntimeSettingsUpdate
): Promise<RuntimeSettingsRecord> {
	return requestJson<RuntimeSettingsRecord>('/settings/runtime', jsonRequest('runtime', 'PUT', payload));
}

export function getStorageSettings(): Promise<StorageSettingsRecord> {
	return requestJson<StorageSettingsRecord>('/settings/storage');
}

export function updateStorageSettings(
	payload: StorageSettingsUpdate
): Promise<StorageSettingsRecord> {
	return requestJson<StorageSettingsRecord>('/settings/storage', jsonRequest('storage', 'PUT', payload));
}

export function getImportSettings(): Promise<ImportSettingsRecord> {
	return requestJson<ImportSettingsRecord>('/settings/import');
}

export function updateImportSettings(payload: ImportSettingsUpdate): Promise<ImportSettingsRecord> {
	return requestJson<ImportSettingsRecord>('/settings/import', jsonRequest('import', 'PUT', payload));
}

export function getAcquisitionSettings(): Promise<AcquisitionSettingsRecord> {
	return requestJson<AcquisitionSettingsRecord>('/settings/acquisition');
}

export function getNotificationSettings(): Promise<NotificationSettingsRecord> {
	return requestJson<NotificationSettingsRecord>('/settings/notifications');
}

export function updateNotificationSettings(
	payload: NotificationSettingsUpdate
): Promise<NotificationSettingsRecord> {
	return requestJson<NotificationSettingsRecord>(
		'/settings/notifications',
		jsonRequest('notifications', 'PUT', payload)
	);
}

export function updateAcquisitionSettings(
	payload: AcquisitionSettingsUpdate
): Promise<AcquisitionSettingsRecord> {
	return requestJson<AcquisitionSettingsRecord>(
		'/settings/acquisition',
		jsonRequest('acquisition', 'PUT', payload)
	);
}

export function getQbittorrentSettings(): Promise<QbittorrentSettingsRecord> {
	return requestJson<QbittorrentSettingsRecord>('/settings/download-clients/qbittorrent');
}

export function updateQbittorrentSettings(
	payload: QbittorrentSettingsUpdate
): Promise<QbittorrentSettingsRecord> {
	return requestJson<QbittorrentSettingsRecord>(
		'/settings/download-clients/qbittorrent',
		jsonRequest('qbittorrent', 'PUT', payload)
	);
}

export function testQbittorrentSettings(
	payload: QbittorrentSettingsUpdate
): Promise<ConnectionTestResult> {
	return requestJson<ConnectionTestResult>(
		'/settings/download-clients/qbittorrent/test',
		jsonRequest('qbittorrent-test', 'POST', payload)
	);
}

export function getProwlarrSettings(): Promise<ProwlarrIntegrationRecord> {
	return requestJson<ProwlarrIntegrationRecord>('/settings/integrations/prowlarr');
}

export function updateProwlarrSettings(
	payload: ProwlarrIntegrationUpdate
): Promise<ProwlarrIntegrationRecord> {
	return requestJson<ProwlarrIntegrationRecord>(
		'/settings/integrations/prowlarr',
		jsonRequest('prowlarr', 'PUT', payload)
	);
}

export function testProwlarrSettings(
	payload: ProwlarrIntegrationUpdate
): Promise<ConnectionTestResult> {
	return requestJson<ConnectionTestResult>(
		'/settings/integrations/prowlarr/test',
		jsonRequest('prowlarr-test', 'POST', payload)
	);
}

export function getAudiobookshelfSettings(): Promise<AudiobookshelfIntegrationRecord> {
	return requestJson<AudiobookshelfIntegrationRecord>('/settings/integrations/audiobookshelf');
}

export function updateAudiobookshelfSettings(
	payload: AudiobookshelfIntegrationUpdate
): Promise<AudiobookshelfIntegrationRecord> {
	return requestJson<AudiobookshelfIntegrationRecord>(
		'/settings/integrations/audiobookshelf',
		jsonRequest('audiobookshelf', 'PUT', payload)
	);
}

export function testAudiobookshelfSettings(
	payload: AudiobookshelfIntegrationUpdate
): Promise<ConnectionTestResult> {
	return requestJson<ConnectionTestResult>(
		'/settings/integrations/audiobookshelf/test',
		jsonRequest('audiobookshelf-test', 'POST', payload)
	);
}

export function listSyncedIndexers(): Promise<SyncedIndexerRecord[]> {
	return requestJson<SyncedIndexerRecord[]>('/settings/synced-indexers');
}

export function triggerLibraryScan(): Promise<LibraryScanResponse> {
	return requestJson<LibraryScanResponse>('/library/scan', { method: 'POST' });
}

export function getLibraryScanStatus(): Promise<LibraryScanJobRecord | null> {
	return requestJson<LibraryScanJobRecord | null>('/library/scan-status');
}
