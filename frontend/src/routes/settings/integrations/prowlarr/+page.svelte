<script lang="ts">
	import { onMount } from 'svelte';
	import { Cable, LoaderCircle, Save, TestTube2 } from 'lucide-svelte';
	import {
		getProwlarrSettings,
		listSyncedIndexers,
		testProwlarrSettings,
		updateProwlarrSettings
	} from '$lib/api';
	import type { SyncedIndexerRecord } from '$lib/types/SyncedIndexerRecord';

	let loading = $state(true);
	let saving = $state(false);
	let testing = $state(false);
	let error = $state('');
	let success = $state('');
	let testResult = $state('');

	let enabled = $state(false);
	let syncEnabled = $state(false);
	let baseUrl = $state('');
	let hasApiKey = $state(false);
	let apiKey = $state('');
	let clearApiKey = $state(false);
	let selectedIndexerIds = $state([] as number[]);
	let syncedIndexers = $state([] as SyncedIndexerRecord[]);

	function toggleIndexer(indexerId: number, checked: boolean) {
		if (checked) {
			if (!selectedIndexerIds.includes(indexerId)) {
				selectedIndexerIds = [...selectedIndexerIds, indexerId];
			}
			return;
		}

		selectedIndexerIds = selectedIndexerIds.filter((id) => id !== indexerId);
	}

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const [settings, indexers] = await Promise.all([getProwlarrSettings(), listSyncedIndexers()]);
			enabled = settings.enabled;
			syncEnabled = settings.sync_enabled;
			baseUrl = settings.base_url;
			hasApiKey = settings.has_api_key;
			selectedIndexerIds = settings.selected_indexer_ids.map(Number);
			syncedIndexers = indexers;
			apiKey = '';
			clearApiKey = false;
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Prowlarr settings failed to load.';
		} finally {
			loading = false;
		}
	}

	function payload() {
		return {
			enabled,
			sync_enabled: syncEnabled,
			base_url: baseUrl,
			api_key: apiKey || null,
			selected_indexer_ids: selectedIndexerIds,
			clear_api_key: clearApiKey
		};
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			const settings = await updateProwlarrSettings(payload());
			hasApiKey = settings.has_api_key;
			apiKey = '';
			clearApiKey = false;
			success = 'Prowlarr settings saved.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Prowlarr settings failed to save.';
		} finally {
			saving = false;
		}
	}

	async function testConnection() {
		testing = true;
		error = '';
		testResult = '';

		try {
			const result = await testProwlarrSettings(payload());
			testResult = result.message;
		} catch (testError) {
			error = testError instanceof Error ? testError.message : 'Prowlarr connection test failed.';
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
			<Cable class="h-4 w-4" />
			<span>Integration</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Prowlarr</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			Save Athena’s outbound Prowlarr connection settings here, then let stock Prowlarr push
			indexers into Athena through the Readarr-compatible application sync surface. Athena will only
			search the selected trackers when you limit the list below.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading Prowlarr settings…</span>
		</div>
	{:else}
		<form
			class="dashboard-card space-y-5"
			onsubmit={(event) => {
				event.preventDefault();
				void saveSettings();
			}}
		>
			<div class="grid gap-4 md:grid-cols-2">
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<input bind:checked={enabled} class="h-4 w-4" type="checkbox" />
					<span class="text-sm font-medium text-stone-800">Enable Prowlarr integration</span>
				</label>
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<input bind:checked={syncEnabled} class="h-4 w-4" type="checkbox" />
					<span class="text-sm font-medium text-stone-800">Enable application sync</span>
				</label>
				<label class="space-y-2 md:col-span-2">
					<span class="text-sm font-medium text-stone-700">Base URL</span>
					<input bind:value={baseUrl} class="input-shell" placeholder="http://localhost:9696" />
				</label>
				<label class="space-y-2 md:col-span-2">
					<span class="text-sm font-medium text-stone-700">API key</span>
					<input
						bind:value={apiKey}
						class="input-shell"
						placeholder={hasApiKey
							? 'Stored API key will be kept unless replaced'
							: 'Enter Prowlarr API key'}
						type="password"
					/>
				</label>
				<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<input bind:checked={clearApiKey} class="h-4 w-4" type="checkbox" />
					<span class="text-sm text-stone-700">
						Clear saved API key {hasApiKey ? '(key currently stored)' : '(no key stored)'}
					</span>
				</label>
				<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-600">
					{#if hasApiKey}
						A Prowlarr API key is stored and stays masked in the UI.
					{:else}
						No Prowlarr API key is currently stored.
					{/if}
				</div>
				<div class="space-y-3 md:col-span-2">
					<div>
						<span class="text-sm font-medium text-stone-700">Search indexers</span>
						<p class="mt-1 text-sm text-stone-600">
							Leave all unchecked to search every synced Prowlarr indexer.
						</p>
					</div>
					{#if syncedIndexers.length === 0}
						<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-600">
							No synced indexers are available yet. Save and sync indexers from Prowlarr first.
						</div>
					{:else}
						<div class="grid gap-3 md:grid-cols-2">
							{#each syncedIndexers as indexer}
								{@const prowlarrIndexerId = indexer.prowlarr_indexer_id
									? Number(indexer.prowlarr_indexer_id)
									: null}
								<label class="flex items-center gap-3 rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
									<input
										checked={prowlarrIndexerId !== null &&
											selectedIndexerIds.includes(prowlarrIndexerId)}
										class="h-4 w-4"
										disabled={prowlarrIndexerId === null}
										onchange={(event) => {
											if (prowlarrIndexerId !== null) {
												toggleIndexer(
													prowlarrIndexerId,
													(event.currentTarget as HTMLInputElement).checked
												);
											}
										}}
										type="checkbox"
									/>
									<span class="text-sm text-stone-800">
										{indexer.name}
										{#if indexer.prowlarr_indexer_id !== null}
											<span class="text-stone-500">
												(ID {String(indexer.prowlarr_indexer_id)})
											</span>
										{/if}
									</span>
								</label>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			{#if error}
				<div
					class="rounded-[1.35rem] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
				>
					{error}
				</div>
			{/if}
			{#if success}
				<div
					class="rounded-[1.35rem] border border-teal-200 bg-teal-50 px-4 py-3 text-sm text-teal-900"
				>
					{success}
				</div>
			{/if}
			{#if testResult}
				<div
					class="rounded-[1.35rem] border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900"
				>
					{testResult}
				</div>
			{/if}

			<div class="flex flex-wrap gap-3">
				<button class="action-button" disabled={saving} type="submit">
					<Save class="h-4 w-4" />
					<span>{saving ? 'Saving…' : 'Save Prowlarr'}</span>
				</button>
				<button class="ghost-button" disabled={testing} onclick={testConnection} type="button">
					<TestTube2 class="h-4 w-4" />
					<span>{testing ? 'Testing…' : 'Test connection'}</span>
				</button>
			</div>
		</form>
	{/if}
</div>
