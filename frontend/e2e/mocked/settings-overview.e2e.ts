import { expect, test } from '@playwright/test';

import { mockAthenaApi } from '../support/api';

test('renders the settings overview from the browser against mocked API responses', async ({
	page
}) => {
	await mockAthenaApi(page);

	await page.goto('/settings');

	await expect(page.getByRole('heading', { level: 1, name: 'Admin settings center' })).toBeVisible();
	await expect(page.getByText('http://localhost:8080')).toBeVisible();
	await expect(page.getByText('http://localhost:9696')).toBeVisible();
	await expect(page.getByText('/ebooks')).toBeVisible();
});

