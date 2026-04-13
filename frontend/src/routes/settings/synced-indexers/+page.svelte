<script lang="ts">
	import { onMount } from 'svelte';
	import { FolderTree, LoaderCircle } from 'lucide-svelte';
	import { listSyncedIndexers } from '$lib/api';
	import type { SyncedIndexerRecord } from '$lib/types/SyncedIndexerRecord';

	let loading = $state(true);
	let error = $state('');
	let indexers = $state([] as SyncedIndexerRecord[]);

	async function loadIndexers() {
		loading = true;
		error = '';

		try {
			indexers = await listSyncedIndexers();
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'Synced indexers failed to load.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadIndexers();
	});
</script>

<div class="space-y-6">
	<div class="dashboard-card">
		<p class="eyebrow">
			<FolderTree class="h-4 w-4" />
			<span>Synced indexers</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Prowlarr-managed indexers</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			This list is read-only in Athena for now. Prowlarr owns create, update, and delete through
			the Readarr-compatible application API.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading synced indexers…</span>
		</div>
	{:else if error}
		<div class="dashboard-card border border-red-200 bg-red-50 text-sm text-red-700">{error}</div>
	{:else if indexers.length === 0}
		<div class="dashboard-card text-sm text-stone-600">
			No indexers have been pushed from Prowlarr yet.
		</div>
	{:else}
		<div class="grid gap-3">
			{#each indexers as indexer}
				<div class="dashboard-card space-y-3">
					<div class="flex flex-wrap items-center gap-2">
						<span class="status-pill">{indexer.enabled ? 'enabled' : 'disabled'}</span>
						{#if indexer.protocol}
							<span class="status-pill bg-white">{indexer.protocol}</span>
						{/if}
						<span class="status-pill bg-white">{indexer.implementation}</span>
					</div>
					<div>
						<h2 class="font-serif text-2xl text-stone-950">{indexer.name}</h2>
						<p class="mt-2 text-sm text-stone-600">
							{indexer.base_url ?? 'No base URL available'}
						</p>
					</div>
					<div class="grid gap-3 md:grid-cols-3">
						<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-700">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Prowlarr indexer ID</p>
							<p class="mt-2">{indexer.prowlarr_indexer_id ?? 'Unknown'}</p>
						</div>
						<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-700">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Categories</p>
							<p class="mt-2">{indexer.categories.join(', ') || 'None'}</p>
						</div>
						<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-700">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Last sync</p>
							<p class="mt-2">{indexer.last_synced_at}</p>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
