import { expect, test } from '@playwright/test';

test('persists qBittorrent settings through the live backend', async ({ page }) => {
	await page.goto('/settings/download-clients/qbittorrent');

	await expect(page.getByRole('button', { name: 'Save qBittorrent' })).toBeVisible();
	await page.getByLabel('Base URL').fill('http://qbittorrent.internal:8080');
	await page.getByLabel('Username').fill('athena-admin');
	await page.getByRole('textbox', { name: 'Password', exact: true }).fill('secret');
	await page.getByRole('button', { name: 'Save qBittorrent' }).click();

	await expect(page.getByText('qBittorrent settings saved.')).toBeVisible();

	await page.reload();

	await expect(page.getByLabel('Base URL')).toHaveValue('http://qbittorrent.internal:8080');
	await expect(page.getByLabel('Username')).toHaveValue('athena-admin');
});
