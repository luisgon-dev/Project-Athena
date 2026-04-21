import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ImportSettingsPage from '../import/+page.svelte';

describe('import settings page', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('loads media-specific import settings and toggles ebook controls by mode', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify({
						ebook_import_mode: 'managed',
						ebook_passthrough_root: null,
						ebook_naming_template: '{author}/{title}/{title}',
						audiobook_layout_preset: 'author_title',
						calibre_command: 'calibredb'
					}),
					{ status: 200, headers: { 'content-type': 'application/json' } }
				)
			)
		);

		render(ImportSettingsPage);

		await expect.element(page.getByText('Ebook naming template')).toBeInTheDocument();
		const modeSelect = document.querySelector('select') as HTMLSelectElement;
		modeSelect.value = 'passthrough';
		modeSelect.dispatchEvent(new Event('change', { bubbles: true }));
		await expect.element(page.getByText('Ebook passthrough directory')).toBeInTheDocument();
		await expect
			.element(page.getByText('Audiobook folder layout', { exact: true }))
			.toBeInTheDocument();
	});
});
