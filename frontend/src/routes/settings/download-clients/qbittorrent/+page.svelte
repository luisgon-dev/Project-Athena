<script lang="ts">
	import { onMount } from 'svelte';
	import { LoaderCircle, Save, TestTube2 } from 'lucide-svelte';
	import {
		getQbittorrentSettings,
		testQbittorrentSettings,
		updateQbittorrentSettings
	} from '$lib/api';

	let loading = $state(true);
	let saving = $state(false);
	let testing = $state(false);
	let error = $state('');
	let success = $state('');
	let testResult = $state('');

	let enabled = $state(false);
	let baseUrl = $state('');
	let username = $state('');
	let categoryEbook = $state('athena-ebooks');
	let categoryAudiobook = $state('athena-audiobooks');
	let hasPassword = $state(false);
	let password = $state('');
	let clearPassword = $state(false);

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const settings = await getQbittorrentSettings();
			enabled = settings.enabled;
			baseUrl = settings.base_url;
			username = settings.username;
			categoryEbook = settings.category_ebook;
			categoryAudiobook = settings.category_audiobook;
			hasPassword = settings.has_password;
			password = '';
			clearPassword = false;
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'qBittorrent settings failed to load.';
		} finally {
			loading = false;
		}
	}

	function payload() {
		return {
			enabled,
			base_url: baseUrl,
			username,
			password: password || null,
			clear_password: clearPassword,
			category_ebook: categoryEbook,
			category_audiobook: categoryAudiobook
		};
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			const settings = await updateQbittorrentSettings(payload());
			hasPassword = settings.has_password;
			password = '';
			clearPassword = false;
			success = 'qBittorrent settings saved.';
		} catch (saveError) {
			error =
				saveError instanceof Error ? saveError.message : 'qBittorrent settings failed to save.';
		} finally {
			saving = false;
		}
	}

	async function testConnection() {
		testing = true;
		error = '';
		testResult = '';

		try {
			const result = await testQbittorrentSettings(payload());
			testResult = result.message;
		} catch (testError) {
			error =
				testError instanceof Error ? testError.message : 'qBittorrent connection test failed.';
		} finally {
			testing = false;
		}
	}

	onMount(() => {
		void loadSettings();
	});
</script>

<div class="space-y-6">
	<div class="dashboard-card">
		<p class="eyebrow">Download client</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">qBittorrent</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			Athena logs into the qBittorrent WebUI, tags dispatches with the request UUID, and tracks
			completions by category. Keep the saved password masked and replace it only when needed.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading qBittorrent settings…</span>
		</div>
	{:else}
		<form
			class="dashboard-card space-y-5"
			onsubmit={(event) => {
				event.preventDefault();
				void saveSettings();
			}}
		>
			<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
				<input bind:checked={enabled} class="h-4 w-4" type="checkbox" />
				<span class="text-sm font-medium text-stone-800">Enable qBittorrent integration</span>
			</label>

			<div class="grid gap-4 md:grid-cols-2">
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Base URL</span>
					<input bind:value={baseUrl} class="input-shell" placeholder="http://localhost:8080" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Username</span>
					<input bind:value={username} class="input-shell" placeholder="admin" />
				</label>
				<label class="space-y-2 md:col-span-2">
					<span class="text-sm font-medium text-stone-700">Password</span>
					<input bind:value={password} class="input-shell" placeholder={hasPassword ? 'Stored password will be kept unless replaced' : 'Enter qBittorrent password'} type="password" />
				</label>
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<input bind:checked={clearPassword} class="h-4 w-4" type="checkbox" />
					<span class="text-sm text-stone-700">
						Clear saved password {hasPassword ? '(password currently stored)' : '(no password stored)'}
					</span>
				</label>
				<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-600">
					{#if hasPassword}
						A password is already stored and remains masked in the UI.
					{:else}
						No qBittorrent password is currently stored.
					{/if}
				</div>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Ebook category</span>
					<input bind:value={categoryEbook} class="input-shell" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Audiobook category</span>
					<input bind:value={categoryAudiobook} class="input-shell" />
				</label>
			</div>

			{#if error}
				<div class="rounded-[1.35rem] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
					{error}
				</div>
			{/if}
			{#if success}
				<div class="rounded-[1.35rem] border border-teal-200 bg-teal-50 px-4 py-3 text-sm text-teal-900">
					{success}
				</div>
			{/if}
			{#if testResult}
				<div class="rounded-[1.35rem] border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
					{testResult}
				</div>
			{/if}

			<div class="flex flex-wrap gap-3">
				<button class="action-button" disabled={saving} type="submit">
					<Save class="h-4 w-4" />
					<span>{saving ? 'Saving…' : 'Save qBittorrent'}</span>
				</button>
				<button class="ghost-button" disabled={testing} onclick={testConnection} type="button">
					<TestTube2 class="h-4 w-4" />
					<span>{testing ? 'Testing…' : 'Test connection'}</span>
				</button>
			</div>
		</form>
	{/if}
</div>
