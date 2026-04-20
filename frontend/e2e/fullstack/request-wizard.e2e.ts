import { expect, test } from '@playwright/test';

test('creates a request through the live backend and opens its detail page', async ({ page }) => {
	await page.goto('/requests/new');

	await page.getByRole('textbox', { name: 'Title', exact: true }).fill('The Hobbit');
	await page.getByLabel('Author').fill('Tolkien');
	await page.getByRole('button', { name: 'Search metadata' }).click();

	await expect(page.getByRole('heading', { level: 4, name: 'The Hobbit' })).toBeVisible();

	await page.getByLabel('Audiobook').check();
	await page.getByLabel('Preferred narrator').fill('Andy Serkis');
	await page.getByRole('button', { name: 'Create request batch' }).click();

	await expect(page.getByRole('heading', { level: 3, name: 'Created requests' })).toBeVisible();

	const requestLink = page.getByRole('link', { name: /The Hobbit/i }).first();
	await requestLink.click();

	await expect(page.getByRole('heading', { level: 1, name: 'The Hobbit' })).toBeVisible();
	await expect(page.getByText('J.R.R. Tolkien', { exact: true }).first()).toBeVisible();
	await expect(page.getByText('Created', { exact: true }).first()).toBeVisible();
});
