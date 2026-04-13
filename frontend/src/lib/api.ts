import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
import type { AcquisitionSettingsRecord } from '$lib/types/AcquisitionSettingsRecord';
import type { AcquisitionSettingsUpdate } from '$lib/types/AcquisitionSettingsUpdate';
import type { ConnectionTestResult } from '$lib/types/ConnectionTestResult';
import type { ImportSettingsRecord } from '$lib/types/ImportSettingsRecord';
import type { ImportSettingsUpdate } from '$lib/types/ImportSettingsUpdate';
import type { ProwlarrIntegrationRecord } from '$lib/types/ProwlarrIntegrationRecord';
import type { ProwlarrIntegrationUpdate } from '$lib/types/ProwlarrIntegrationUpdate';
import type { QbittorrentSettingsRecord } from '$lib/types/QbittorrentSettingsRecord';
import type { QbittorrentSettingsUpdate } from '$lib/types/QbittorrentSettingsUpdate';
import type { RequestDetailRecord } from '$lib/types/RequestDetailRecord';
import type { RequestListRecord } from '$lib/types/RequestListRecord';
import type { RequestRecord } from '$lib/types/RequestRecord';
import type { RuntimeSettingsRecord } from '$lib/types/RuntimeSettingsRecord';
import type { RuntimeSettingsUpdate } from '$lib/types/RuntimeSettingsUpdate';
import type { StorageSettingsRecord } from '$lib/types/StorageSettingsRecord';
import type { StorageSettingsUpdate } from '$lib/types/StorageSettingsUpdate';
import type { SyncedIndexerRecord } from '$lib/types/SyncedIndexerRecord';
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

export function getRequestDetail(requestId: string): Promise<RequestDetailRecord> {
	return requestJson<RequestDetailRecord>(`/requests/${requestId}`);
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

export function listSyncedIndexers(): Promise<SyncedIndexerRecord[]> {
	return requestJson<SyncedIndexerRecord[]>('/settings/synced-indexers');
}
