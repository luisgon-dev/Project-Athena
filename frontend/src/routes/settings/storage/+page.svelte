<script lang="ts">
	import { onMount } from 'svelte';
	import { Database, LoaderCircle, Save } from 'lucide-svelte';
	import { getStorageSettings, updateStorageSettings } from '$lib/api';

	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');

	let ebooksRoot = $state('');
	let audiobooksRoot = $state('');

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const settings = await getStorageSettings();
			ebooksRoot = settings.ebooks_root;
			audiobooksRoot = settings.audiobooks_root;
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Storage settings failed to load.';
		} finally {
			loading = false;
		}
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			await updateStorageSettings({
				ebooks_root: ebooksRoot,
				audiobooks_root: audiobooksRoot
			});
			success = 'Storage settings saved.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Storage settings failed to save.';
		} finally {
			saving = false;
		}
	}

	onMount(() => {
		void loadSettings();
	});
</script>

<div class="space-y-6">
	<div class="dashboard-card">
		<p class="eyebrow">
			<Database class="h-4 w-4" />
			<span>Storage</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Media roots</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			These paths are validated as absolute directories and applied immediately to import and
			file placement flows.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading storage settings…</span>
		</div>
	{:else}
		<form
			class="dashboard-card space-y-5"
			onsubmit={(event) => {
				event.preventDefault();
				void saveSettings();
			}}
		>
			<div class="grid gap-4">
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Ebooks root</span>
					<input bind:value={ebooksRoot} class="input-shell" placeholder="/ebooks" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Audiobooks root</span>
					<input bind:value={audiobooksRoot} class="input-shell" placeholder="/audiobooks" />
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

			<button class="action-button" disabled={saving} type="submit">
				<Save class="h-4 w-4" />
				<span>{saving ? 'Saving…' : 'Save storage'}</span>
			</button>
		</form>
	{/if}
</div>
