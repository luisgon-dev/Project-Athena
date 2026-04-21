import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import SettingsOverview from '../+page.svelte';

describe('settings overview page', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('renders the admin settings summary from the JSON api', async () => {
		vi.stubGlobal(
			'fetch',
			vi
				.fn()
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify({
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
									has_api_key: true
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
								blocked_terms: []
							}
						}),
						{ status: 200, headers: { 'content-type': 'application/json' } }
					)
				)
				.mockResolvedValueOnce(
					new Response(JSON.stringify([{ id: 1 }]), {
						status: 200,
						headers: { 'content-type': 'application/json' }
					})
				)
		);

		render(SettingsOverview);

		await expect
			.element(page.getByRole('heading', { level: 1 }))
			.toHaveTextContent('Admin settings center');
		await expect.element(page.getByText('http://localhost:8080')).toBeInTheDocument();
		await expect.element(page.getByText('http://localhost:9696')).toBeInTheDocument();
		await expect.element(page.getByText('/ebooks')).toBeInTheDocument();
	});
});
