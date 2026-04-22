import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { authState } from '$lib/auth';
import DashboardPage from '../+page.svelte';

describe('dashboard page', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('renders the dashboard with request data from the JSON API', async () => {
		authState.set({
			loading: false,
			setupRequired: false,
			user: { id: 'admin-1', username: 'admin', role: 'admin' }
		});

		vi.stubGlobal(
			'fetch',
			vi
				.fn()
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify([
							{
								id: 'sub-1',
								requested_by_user_id: 'user-1',
								requested_by_username: 'alice',
								intake_mode: 'metadata',
								title: 'The Hobbit',
								author: 'J.R.R. Tolkien',
								external_work_id: 'OL1',
								notes: null,
								media_types: ['Audiobook'],
								preferred_language: 'en',
								manifestation: {
									edition_title: null,
									preferred_narrator: null,
									preferred_publisher: null,
									graphic_audio: false
								},
								status: 'submitted',
								requires_admin_approval: true,
								allow_duplicate: false,
								linked_requests: [],
								created_at: '2026-04-10 00:00:00',
								updated_at: '2026-04-10 00:00:00'
							}
						]),
						{ status: 200, headers: { 'content-type': 'application/json' } }
					)
				)
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify([
							{ id: 'req-1', title: 'The Hobbit', author: 'J.R.R. Tolkien', media_type: 'Ebook', state: 'requested', created_at: '2026-04-10 00:00:00' },
							{ id: 'req-2', title: 'Dune', author: 'Frank Herbert', media_type: 'Audiobook', state: 'imported', created_at: '2026-04-10 00:01:00' }
						]),
						{ status: 200, headers: { 'content-type': 'application/json' } }
					)
				)
		);

		render(DashboardPage);

		await expect.element(page.getByRole('heading', { level: 1 })).toHaveTextContent(
			'Submission and fulfillment queue'
		);
		await expect.element(page.getByRole('link', { name: /submitted The Hobbit/i })).toBeInTheDocument();
		await expect.element(page.getByText('Dune')).toBeInTheDocument();
	});
});
