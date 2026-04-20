import { expect, test } from '@playwright/test';

import { jsonResponse, mockAthenaApi } from '../support/api';

test('searches metadata and creates a request batch', async ({ page }) => {
	await mockAthenaApi(page, {
		'POST /api/v1/requests': jsonResponse(
			[
				{
					id: 'req-created-1',
					external_work_id: 'OL82563W',
					title: 'The Hobbit',
					author: 'J.R.R. Tolkien',
					media_type: 'Ebook',
					preferred_language: 'en',
					manifestation: {
						edition_title: 'Unabridged Edition',
						preferred_narrator: 'Andy Serkis',
						preferred_publisher: 'HarperCollins',
						graphic_audio: false
					},
					state: 'requested',
					created_at: '2026-04-20 10:10:00'
				}
			],
			201
		)
	});

	await page.goto('/requests/new');

	await page.getByRole('textbox', { name: 'Title', exact: true }).fill('The Hobbit');
	await page.getByLabel('Author').fill('Tolkien');
	await page.getByRole('button', { name: 'Search metadata' }).click();

	await expect(page.getByRole('heading', { level: 4, name: 'The Hobbit' })).toBeVisible();

	await page.getByLabel('Preferred narrator').fill('Andy Serkis');
	await page.getByRole('button', { name: 'Create request batch' }).click();

	await expect(page.getByRole('heading', { level: 3, name: 'Created requests' })).toBeVisible();
	await expect(page.getByRole('link', { name: /The Hobbit/i })).toBeVisible();
});

test('shows a validation error when the backend rejects request creation', async ({ page }) => {
	await mockAthenaApi(page, {
		'POST /api/v1/requests': jsonResponse({ error: 'missing selected_work_id' }, 400)
	});

	await page.goto('/requests/new');

	await page.getByRole('button', { name: 'Create request batch' }).click();

	await expect(page.getByText('missing selected_work_id')).toBeVisible();
});
