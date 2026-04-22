import type { Page, Route } from '@playwright/test';

type JsonResponder = {
	status?: number;
	body?: unknown;
	headers?: Record<string, string>;
};

type MockResponder = JsonResponder | ((route: Route) => Promise<void>);

type MockOverrides = Record<string, MockResponder>;

function json(route: Route, responder: JsonResponder = {}) {
	return route.fulfill({
		status: responder.status ?? 200,
		contentType: 'application/json',
		headers: responder.headers,
		body: JSON.stringify(responder.body ?? null)
	});
}

function requestRecord(overrides: Record<string, unknown> = {}) {
	return {
		id: 'req-1',
		external_work_id: 'OL82563W',
		title: 'The Hobbit',
		author: 'J.R.R. Tolkien',
		media_type: 'Audiobook',
		preferred_language: 'en',
		manifestation: {
			edition_title: 'Unabridged Edition',
			preferred_narrator: 'Andy Serkis',
			preferred_publisher: 'HarperCollins',
			graphic_audio: false
		},
		state: 'review',
		created_at: '2026-04-20 10:00:00',
		...overrides
	};
}

function requestDetail(
	overrides: {
		request?: Record<string, unknown>;
		review_queue?: Array<Record<string, unknown>>;
		events?: Array<Record<string, unknown>>;
	} = {}
) {
	const request = requestRecord(overrides.request);
	return {
		request,
		review_queue: overrides.review_queue?.map((entry) => ({
			id: 1,
			request_id: request.id,
			candidate: {
				external_id: 'cand-1',
				source: 'Prowlarr',
				title: 'The Hobbit by J.R.R. Tolkien',
				protocol: 'torrent',
				size_bytes: 987654321,
				indexer: 'indexer-1',
				download_url: 'magnet:?xt=urn:btih:abcdef',
				narrator: 'Andy Serkis',
				graphic_audio: false,
				detected_language: 'en'
			},
			score: 0.97,
			explanation: ['Exact title match', 'Preferred narrator matched'],
			created_at: '2026-04-20 10:01:00',
			...entry
		})) ?? [
			{
				id: 1,
				request_id: request.id,
				candidate: {
					external_id: 'cand-1',
					source: 'Prowlarr',
					title: 'The Hobbit by J.R.R. Tolkien',
					protocol: 'torrent',
					size_bytes: 987654321,
					indexer: 'indexer-1',
					download_url: 'magnet:?xt=urn:btih:abcdef',
					narrator: 'Andy Serkis',
					graphic_audio: false,
					detected_language: 'en'
				},
				score: 0.97,
				explanation: ['Exact title match', 'Preferred narrator matched'],
				created_at: '2026-04-20 10:01:00'
			}
		],
		events: overrides.events?.map((event) => ({
			id: 1,
			request_id: request.id,
			kind: 'Created',
			payload_json: JSON.stringify({
				title: request.title,
				author: request.author,
				media_type: request.media_type
			}),
			created_at: '2026-04-20 10:00:00',
			...event
		})) ?? [
			{
				id: 1,
				request_id: request.id,
				kind: 'Created',
				payload_json: JSON.stringify({
					title: request.title,
					author: request.author,
					media_type: request.media_type
				}),
				created_at: '2026-04-20 10:00:00'
			},
			{
				id: 2,
				request_id: request.id,
				kind: 'ReviewQueued',
				payload_json: JSON.stringify({
					queued_candidates: 1
				}),
				created_at: '2026-04-20 10:01:00'
			}
		]
	};
}

function runtimeSettings() {
	return {
		storage: { ebooks_root: '/ebooks', audiobooks_root: '/audiobooks' },
		metadata: {
			base_url: 'https://openlibrary.org',
			cover_base_url: 'https://covers.openlibrary.org'
		},
		download_clients: {
			qbittorrent: {
				enabled: true,
				base_url: 'http://localhost:8080',
				username: 'admin',
				category_ebook: 'athena-ebooks',
				category_audiobook: 'athena-audiobooks',
				has_password: true
			}
		},
		integrations: {
			prowlarr: {
				enabled: true,
				sync_enabled: true,
				base_url: 'http://localhost:9696',
				has_api_key: true,
				selected_indexer_ids: [42]
			},
			audiobookshelf: {
				enabled: true,
				base_url: 'http://localhost:13378',
				library_id: 'library-1',
				has_api_key: true,
				mark_existing_during_search: true
			}
		},
		import: {
			ebook_import_mode: 'managed',
			ebook_passthrough_root: null,
			ebook_naming_template: '{author}/{title}/{title}',
			audiobook_layout_preset: 'author_title',
			calibre_command: 'calibredb'
		},
		acquisition: {
			minimum_score: 0.7,
			auto_acquire_score: 0.93,
			preferred_language: 'en',
			blocked_terms: ['abridged']
		},
		notifications: {
			enabled: false,
			target_kind: 'webhook',
			target_url: '',
			has_auth_token: false,
			auth_header: null
		}
	};
}

const defaultRequests = [
	{
		id: 'req-1',
		title: 'The Hobbit',
		author: 'J.R.R. Tolkien',
		media_type: 'Audiobook',
		state: 'review',
		created_at: '2026-04-20 10:00:00'
	},
	{
		id: 'req-2',
		title: 'Dune',
		author: 'Frank Herbert',
		media_type: 'Ebook',
		state: 'imported',
		created_at: '2026-04-20 09:30:00'
	}
];

const defaultSearchResults = {
	works: [
		{
			external_id: 'OL82563W',
			title: 'The Hobbit',
			primary_author: 'J.R.R. Tolkien',
			first_publish_year: 1937,
			description: 'A fantasy adventure in Middle-earth.',
			cover_id: null,
			subjects: ['Fantasy', 'Middle-earth'],
			edition_count: 12
		}
	]
};

function defaultResponder(method: string, pathname: string): MockResponder | undefined {
	switch (`${method} ${pathname}`) {
		case 'GET /api/v1/auth/bootstrap':
			return {
				body: {
					setup_required: false,
					authenticated_user: {
						id: 'admin-1',
						username: 'admin',
						role: 'admin'
					}
				}
			};
		case 'GET /api/v1/requests':
			return { body: defaultRequests };
		case 'GET /api/v1/requests/search':
			return { body: defaultSearchResults };
		case 'POST /api/v1/requests':
			return {
				status: 201,
				body: [
					requestRecord({ id: 'req-created-1', media_type: 'Ebook', state: 'requested' }),
					requestRecord({ id: 'req-created-2', media_type: 'Audiobook', state: 'requested' })
				]
			};
		case 'GET /api/v1/requests/req-1':
			return { body: requestDetail() };
		case 'POST /api/v1/requests/req-1/retry-search':
			return {
				body: requestDetail({
					events: [
						{
							id: 1,
							request_id: 'req-1',
							kind: 'Created',
							payload_json: JSON.stringify({
								title: 'The Hobbit',
								author: 'J.R.R. Tolkien',
								media_type: 'Audiobook'
							}),
							created_at: '2026-04-20 10:00:00'
						},
						{
							id: 3,
							request_id: 'req-1',
							kind: 'SearchCompleted',
							payload_json: JSON.stringify({
								outcome: 'review',
								qualified_candidates: 1,
								top_score: 0.97
							}),
							created_at: '2026-04-20 10:05:00'
						}
					]
				})
			};
		case 'POST /api/v1/requests/req-1/review-queue/1/approve':
			return {
				body: requestDetail({
					request: { state: 'queued_for_download' },
					review_queue: [],
					events: [
						{
							id: 1,
							request_id: 'req-1',
							kind: 'Created',
							payload_json: JSON.stringify({
								title: 'The Hobbit',
								author: 'J.R.R. Tolkien',
								media_type: 'Audiobook'
							}),
							created_at: '2026-04-20 10:00:00'
						},
						{
							id: 4,
							request_id: 'req-1',
							kind: 'ReviewApproved',
							payload_json: JSON.stringify({
								candidate_title: 'The Hobbit by J.R.R. Tolkien',
								candidate_source: 'Prowlarr'
							}),
							created_at: '2026-04-20 10:06:00'
						}
					]
				})
			};
		case 'POST /api/v1/requests/req-1/review-queue/1/reject':
			return {
				body: requestDetail({
					request: { state: 'no_match' },
					review_queue: [],
					events: [
						{
							id: 1,
							request_id: 'req-1',
							kind: 'Created',
							payload_json: JSON.stringify({
								title: 'The Hobbit',
								author: 'J.R.R. Tolkien',
								media_type: 'Audiobook'
							}),
							created_at: '2026-04-20 10:00:00'
						},
						{
							id: 5,
							request_id: 'req-1',
							kind: 'ReviewRejected',
							payload_json: JSON.stringify({
								rejected_candidate_id: 'cand-1'
							}),
							created_at: '2026-04-20 10:07:00'
						}
					]
				})
			};
		case 'GET /api/v1/settings/runtime':
			return { body: runtimeSettings() };
		case 'GET /api/v1/settings/synced-indexers':
			return {
				body: [
					{
						id: 1,
						name: 'Mock Indexer',
						implementation: 'Torznab',
						protocol: 'torrent',
						base_url: 'http://localhost:8088',
						prowlarr_indexer_id: 42,
						categories: ['3030'],
						last_synced_at: '2026-04-20 08:00:00',
						enabled: true
					}
				]
			};
		case 'GET /api/v1/settings/download-clients/qbittorrent':
			return {
				body: {
					enabled: true,
					base_url: 'http://localhost:8080',
					username: 'admin',
					category_ebook: 'athena-ebooks',
					category_audiobook: 'athena-audiobooks',
					has_password: true
				}
			};
		case 'PUT /api/v1/settings/download-clients/qbittorrent':
			return {
				body: {
					enabled: true,
					base_url: 'http://localhost:8080',
					username: 'admin',
					category_ebook: 'athena-ebooks',
					category_audiobook: 'athena-audiobooks',
					has_password: true
				}
			};
		case 'POST /api/v1/settings/download-clients/qbittorrent/test':
			return {
				body: {
					ok: true,
					message: 'qBittorrent connection succeeded'
				}
			};
		case 'POST /api/v1/library/scan':
			return { status: 202, body: { job_id: 9 } };
		case 'GET /api/v1/library/scan-status':
			return {
				body: {
					id: 9,
					ebooks_found: 4,
					audiobooks_found: 2,
					duplicates_skipped: 1,
					started_at: '2026-04-20 11:00:00',
					completed_at: '2026-04-20 11:00:05',
					error_message: null
				}
			};
		default:
			return undefined;
	}
}

export async function mockAthenaApi(page: Page, overrides: MockOverrides = {}) {
	await page.route('**/api/v1/**', async (route) => {
		const request = route.request();
		const url = new URL(request.url());
		const key = `${request.method()} ${url.pathname}`;
		const responder = overrides[key] ?? defaultResponder(request.method(), url.pathname);

		if (!responder) {
			throw new Error(`No mocked Athena response for ${key}`);
		}

		if (typeof responder === 'function') {
			await responder(route);
			return;
		}

		await json(route, responder);
	});
}

export function jsonResponse(body: unknown, status = 200): JsonResponder {
	return { status, body };
}
