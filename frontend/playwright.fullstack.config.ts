import { defineConfig } from '@playwright/test';

export default defineConfig({
	testDir: './e2e/fullstack',
	workers: 1,
	use: {
		baseURL: 'http://127.0.0.1:4174',
		trace: 'retain-on-failure'
	},
	webServer: [
		{
			command: 'node ./scripts/openlibrary-fixture.mjs',
			port: 5001,
			reuseExistingServer: false,
			timeout: 30_000
		},
		{
			command: 'npm run build && ./scripts/start-fullstack-backend.sh',
			port: 4174,
			reuseExistingServer: false,
			timeout: 120_000
		}
	],
	testMatch: '**/*.e2e.{ts,js}'
});
