import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import DashboardPage from '../+page.svelte';

describe('dashboard page', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('renders the dashboard with request data from the JSON API', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
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

		await expect.element(page.getByRole('heading', { level: 1 })).toHaveTextContent('Request Radar');
		await expect.element(page.getByText('The Hobbit')).toBeInTheDocument();
		await expect.element(page.getByText('Dune')).toBeInTheDocument();
	});
});
