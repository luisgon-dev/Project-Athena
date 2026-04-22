<script lang="ts">
	import { onMount } from 'svelte';
	import { Headphones, LoaderCircle, Save, TestTube2 } from 'lucide-svelte';
	import {
		getAudiobookshelfSettings,
		testAudiobookshelfSettings,
		updateAudiobookshelfSettings
	} from '$lib/api';

	let loading = $state(true);
	let saving = $state(false);
	let testing = $state(false);
	let error = $state('');
	let success = $state('');
	let testResult = $state('');

	let enabled = $state(false);
	let baseUrl = $state('');
	let libraryId = $state('');
	let hasApiKey = $state(false);
	let apiKey = $state('');
	let clearApiKey = $state(false);
	let markExistingDuringSearch = $state(false);

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const settings = await getAudiobookshelfSettings();
			enabled = settings.enabled;
			baseUrl = settings.base_url;
			libraryId = settings.library_id;
			hasApiKey = settings.has_api_key;
			markExistingDuringSearch = settings.mark_existing_during_search;
			apiKey = '';
			clearApiKey = false;
		} catch (loadError) {
			error =
				loadError instanceof Error
					? loadError.message
					: 'Audiobookshelf settings failed to load.';
		} finally {
			loading = false;
		}
	}

	function payload() {
		return {
			enabled,
			base_url: baseUrl,
			library_id: libraryId,
			api_key: apiKey || null,
			mark_existing_during_search: markExistingDuringSearch,
			clear_api_key: clearApiKey
		};
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			const settings = await updateAudiobookshelfSettings(payload());
			hasApiKey = settings.has_api_key;
			apiKey = '';
			clearApiKey = false;
			success = 'Audiobookshelf settings saved.';
		} catch (saveError) {
			error =
				saveError instanceof Error
					? saveError.message
					: 'Audiobookshelf settings failed to save.';
		} finally {
			saving = false;
		}
	}

	async function testConnection() {
		testing = true;
		error = '';
		testResult = '';

		try {
			const result = await testAudiobookshelfSettings(payload());
			testResult = result.message;
		} catch (testError) {
			error =
				testError instanceof Error
					? testError.message
					: 'Audiobookshelf connection test failed.';
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
		<p class="eyebrow">
			<Headphones class="h-4 w-4" />
			<span>Integration</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Audiobookshelf</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			Athena uses Audiobookshelf only after an audiobook import succeeds. Save the service URL,
			target library, and API key so the download worker can trigger a library scan on sync and
			optionally mark existing titles during requester search.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading Audiobookshelf settings…</span>
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
				<span class="text-sm font-medium text-stone-800">Enable Audiobookshelf integration</span>
			</label>

			<div class="grid gap-4 md:grid-cols-2">
				<label class="space-y-2 md:col-span-2">
					<span class="text-sm font-medium text-stone-700">Base URL</span>
					<input bind:value={baseUrl} class="input-shell" placeholder="http://localhost:13378" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Library ID</span>
					<input bind:value={libraryId} class="input-shell" placeholder="library-id" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">API key</span>
					<input
						bind:value={apiKey}
						class="input-shell"
						placeholder={hasApiKey ? 'Stored API key will be kept unless replaced' : 'Enter Audiobookshelf API key'}
						type="password"
					/>
				</label>
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<input bind:checked={clearApiKey} class="h-4 w-4" type="checkbox" />
					<span class="text-sm text-stone-700">
						Clear saved API key {hasApiKey ? '(key currently stored)' : '(no key stored)'}
					</span>
				</label>
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4 md:col-span-2">
					<input bind:checked={markExistingDuringSearch} class="h-4 w-4" type="checkbox" />
					<span class="text-sm text-stone-700">
						Use Audiobookshelf to mark existing audiobook matches during requester search
					</span>
				</label>
				<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-600">
					{#if hasApiKey}
						An Audiobookshelf API key is stored and stays masked in the UI.
					{:else}
						No Audiobookshelf API key is currently stored.
					{/if}
				</div>
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
					<span>{saving ? 'Saving…' : 'Save Audiobookshelf'}</span>
				</button>
				<button class="ghost-button" disabled={testing} onclick={testConnection} type="button">
					<TestTube2 class="h-4 w-4" />
					<span>{testing ? 'Testing…' : 'Test connection'}</span>
				</button>
			</div>
		</form>
	{/if}
</div>
