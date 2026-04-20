import { expect, test } from '@playwright/test';

import { jsonResponse, mockAthenaApi } from '../support/api';

test.beforeEach(async ({ page }) => {
	await mockAthenaApi(page);
});

test('renders request data from the Athena API', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { level: 1, name: 'Request Radar' })).toBeVisible();
	await expect(page.getByText('The Hobbit')).toBeVisible();
	await expect(page.getByText('Dune')).toBeVisible();
});

test('shows the dashboard empty state when no requests exist', async ({ page }) => {
	await mockAthenaApi(page, {
		'GET /api/v1/requests': jsonResponse([])
	});

	await page.goto('/');

	await expect(
		page.getByText('No requests exist yet. Start with the metadata wizard and the board will populate on refresh.')
	).toBeVisible();
});

test('shows the dashboard API error state when loading fails', async ({ page }) => {
	await mockAthenaApi(page, {
		'GET /api/v1/requests': jsonResponse({ error: 'Request API offline' }, 503)
	});

	await page.goto('/');

	await expect(page.getByText('Request API offline')).toBeVisible();
});

