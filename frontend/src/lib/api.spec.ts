import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
import { createRequests, getRequestDetail, listRequests, searchRequests } from './api';

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
});
