import { expect, test } from '@playwright/test';

test('renders the empty dashboard against the live backend', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { level: 1, name: 'Request Radar' })).toBeVisible();
	await expect(page.getByRole('link', { name: 'New request' })).toBeVisible();
	await expect(page.getByText('No requests exist yet.', { exact: false })).toBeVisible();
});
