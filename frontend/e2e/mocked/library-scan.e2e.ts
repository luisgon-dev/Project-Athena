import { expect, test } from '@playwright/test';

import { mockAthenaApi } from '../support/api';

test('starts a library scan and renders the completed status', async ({ page }) => {
	await mockAthenaApi(page);

	await page.goto('/library/scan');

	await page.getByRole('button', { name: 'Start scan' }).click();

	await expect(page.getByText('Completed')).toBeVisible();
	await expect(
		page
			.locator('.dashboard-card')
			.filter({ hasText: 'Ebooks found' })
			.getByText('4', { exact: true })
	).toBeVisible();
	await expect(
		page
			.locator('.dashboard-card')
			.filter({ hasText: 'Audiobooks found' })
			.getByText('2', { exact: true })
	).toBeVisible();
	await expect(
		page
			.locator('.dashboard-card')
			.filter({ hasText: 'Duplicates skipped' })
			.getByText('1', { exact: true })
	).toBeVisible();
});
