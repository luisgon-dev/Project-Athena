import { expect, test } from '@playwright/test';

import { mockAthenaApi } from '../support/api';

test('approves a review candidate from the detail page', async ({ page }) => {
	await mockAthenaApi(page);

	await page.goto('/requests/req-1');

	await page.getByRole('heading', { level: 3, name: 'The Hobbit by J.R.R. Tolkien' }).click();
	await page.getByRole('button', { name: 'Approve' }).click();

	await expect(page.getByText('Candidate approved and dispatched to qBittorrent.')).toBeVisible();
	await expect(page.getByText('No candidates are waiting for manual review on this request.')).toBeVisible();
});

test('retries a search when the request is eligible for retry', async ({ page }) => {
	await mockAthenaApi(page, {
		'GET /api/v1/requests/req-1': {
			body: {
				request: {
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
					state: 'no_match',
					created_at: '2026-04-20 10:00:00'
				},
				review_queue: [],
				events: [
					{
						id: 1,
						request_id: 'req-1',
						kind: 'Created',
						payload_json:
							'{"title":"The Hobbit","author":"J.R.R. Tolkien","media_type":"Audiobook"}',
						created_at: '2026-04-20 10:00:00'
					}
				]
			}
		}
	});

	await page.goto('/requests/req-1');
	await page.getByRole('button', { name: 'Retry search' }).click();

	await expect(page.getByText('Search retried with the current acquisition settings.')).toBeVisible();
	await expect(page.getByText('review with 1 qualified candidates; top score 0.97')).toBeVisible();
});

