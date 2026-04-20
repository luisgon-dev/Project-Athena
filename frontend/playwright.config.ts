import { defineConfig } from '@playwright/test';

export default defineConfig({
	testDir: './e2e/mocked',
	use: {
		baseURL: 'http://127.0.0.1:4173',
		trace: 'retain-on-failure'
	},
	webServer: {
		command: 'npm run build && npm run preview -- --host 127.0.0.1 --port 4173',
		port: 4173,
		reuseExistingServer: !process.env.CI,
		timeout: 120_000
	},
	testMatch: '**/*.e2e.{ts,js}'
});
