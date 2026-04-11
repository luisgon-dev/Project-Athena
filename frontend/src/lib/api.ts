import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
import type { RequestDetailRecord } from '$lib/types/RequestDetailRecord';
import type { RequestListRecord } from '$lib/types/RequestListRecord';
import type { RequestRecord } from '$lib/types/RequestRecord';
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
