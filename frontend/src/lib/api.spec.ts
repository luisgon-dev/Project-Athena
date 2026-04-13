import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
import {
	createRequests,
	getQbittorrentSettings,
	getRequestDetail,
	listRequests,
	listSyncedIndexers,
	searchRequests,
	testProwlarrSettings,
	updateStorageSettings
} from './api';

describe('api client', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('lists requests from the phase 2 JSON API', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify([
						{ id: 'req-1', title: 'The Hobbit', author: 'J.R.R. Tolkien', media_type: 'Ebook', state: 'requested', created_at: '2026-04-10 00:00:00' }
					]),
					{ status: 200, headers: { 'content-type': 'application/json' } }
				)
			)
		);

		const requests = await listRequests();

		expect(fetch).toHaveBeenCalledWith('/api/v1/requests', undefined);
		expect(requests[0].title).toBe('The Hobbit');
	});

	it('searches requests metadata with encoded query params', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify({
						works: [{ external_id: 'OL27448W', title: 'The Hobbit', primary_author: 'J.R.R. Tolkien', first_publish_year: 1937, description: null, cover_id: 2468, subjects: ['Fantasy'], edition_count: 42 }]
					}),
					{ status: 200, headers: { 'content-type': 'application/json' } }
				)
			)
		);

		const response = await searchRequests({ title: 'The Hobbit', author: 'Tolkien' });

		expect(fetch).toHaveBeenCalledWith(
			'/api/v1/requests/search?title=The+Hobbit&author=Tolkien',
			undefined
		);
		expect(response.works[0].external_id).toBe('OL27448W');
	});

	it('posts create payloads as JSON', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify([
						{ id: 'req-1', external_work_id: 'OL27448W', title: 'The Hobbit', author: 'J.R.R. Tolkien', media_type: 'Ebook', preferred_language: 'en', manifestation: { edition_title: null, preferred_narrator: null, preferred_publisher: null, graphic_audio: false }, state: 'requested', created_at: '2026-04-10 00:00:00' }
					]),
					{ status: 201, headers: { 'content-type': 'application/json' } }
				)
			)
		);

		const payload: CreateRequestSelection = {
			selected_work_id: 'OL27448W',
			media_types: ['Ebook'],
			preferred_language: 'en',
			manifestation: {
				edition_title: null,
				preferred_narrator: null,
				preferred_publisher: null,
				graphic_audio: false
			}
		};

		const requests = await createRequests(payload);

		expect(fetch).toHaveBeenCalledWith('/api/v1/requests', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});
		expect(requests[0].external_work_id).toBe('OL27448W');
	});

	it('surfaces backend JSON errors', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(JSON.stringify({ error: 'Metadata service timed out' }), {
					status: 504,
					headers: { 'content-type': 'application/json' }
				})
			)
		);

		await expect(getRequestDetail('req-timeout')).rejects.toThrow('Metadata service timed out');
	});

	it('calls section settings endpoints with JSON payloads', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify({ ebooks_root: '/srv/ebooks', audiobooks_root: '/srv/audiobooks' }),
					{ status: 200, headers: { 'content-type': 'application/json' } }
				)
			)
		);

		const response = await updateStorageSettings({
			ebooks_root: '/srv/ebooks',
			audiobooks_root: '/srv/audiobooks'
		});

		expect(fetch).toHaveBeenCalledWith('/api/v1/settings/storage', {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({
				ebooks_root: '/srv/ebooks',
				audiobooks_root: '/srv/audiobooks'
			})
		});
		expect(response.ebooks_root).toBe('/srv/ebooks');
	});

	it('loads qBittorrent settings and synced indexers from the admin api', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn()
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify({
							enabled: true,
							base_url: 'http://localhost:8080',
							username: 'admin',
							category_ebook: 'athena-ebooks',
							category_audiobook: 'athena-audiobooks',
							has_password: true
						}),
						{ status: 200, headers: { 'content-type': 'application/json' } }
					)
				)
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify([
							{
								id: 1,
								prowlarr_indexer_id: 12,
								name: 'Books',
								enabled: true,
								implementation: 'Torznab',
								protocol: 'torrent',
								base_url: 'http://prowlarr.local/12/',
								categories: [3030],
								last_synced_at: '2026-04-11 00:00:00'
							}
						]),
						{ status: 200, headers: { 'content-type': 'application/json' } }
					)
				)
		);

		const qb = await getQbittorrentSettings();
		const synced = await listSyncedIndexers();

		expect(qb.has_password).toBe(true);
		expect(synced[0].prowlarr_indexer_id).toBe(12);
	});

	it('posts prowlarr connection tests', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(JSON.stringify({ ok: true, message: 'Prowlarr connection succeeded' }), {
					status: 200,
					headers: { 'content-type': 'application/json' }
				})
			)
		);

		const result = await testProwlarrSettings({
			enabled: true,
			sync_enabled: true,
			base_url: 'http://localhost:9696',
			api_key: 'secret',
			clear_api_key: false
		});

		expect(fetch).toHaveBeenCalledWith('/api/v1/settings/integrations/prowlarr/test', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({
				enabled: true,
				sync_enabled: true,
				base_url: 'http://localhost:9696',
				api_key: 'secret',
				clear_api_key: false
			})
		});
		expect(result.ok).toBe(true);
	});
});
