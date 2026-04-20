import { expect, test } from '@playwright/test';

import { mockAthenaApi } from '../support/api';

test('loads, saves, and tests qBittorrent settings', async ({ page }) => {
	await mockAthenaApi(page);

	await page.goto('/settings/download-clients/qbittorrent');

	await expect(page.getByRole('heading', { level: 1, name: 'qBittorrent' })).toBeVisible();
	await expect(page.getByLabel('Base URL')).toHaveValue('http://localhost:8080');

	await page.getByRole('button', { name: 'Save qBittorrent' }).click();
	await expect(page.getByText('qBittorrent settings saved.')).toBeVisible();

	await page.getByRole('button', { name: 'Test connection' }).click();
	await expect(page.getByText('qBittorrent connection succeeded')).toBeVisible();
});

