<script lang="ts">
	import { onMount } from 'svelte';
	import { Library, LoaderCircle, Save } from 'lucide-svelte';
	import { getImportSettings, updateImportSettings } from '$lib/api';
	import type { AudiobookLayoutPreset } from '$lib/types/AudiobookLayoutPreset';
	import type { EbookImportMode } from '$lib/types/EbookImportMode';

	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');

	let ebookImportMode = $state('managed' as EbookImportMode);
	let ebookPassthroughRoot = $state('');
	let ebookNamingTemplate = $state('');
	let audiobookLayoutPreset = $state('author_title' as AudiobookLayoutPreset);
	let calibreCommand = $state('');

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const settings = await getImportSettings();
			ebookImportMode = settings.ebook_import_mode;
			ebookPassthroughRoot = settings.ebook_passthrough_root ?? '';
			ebookNamingTemplate = settings.ebook_naming_template;
			audiobookLayoutPreset = settings.audiobook_layout_preset;
			calibreCommand = settings.calibre_command;
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Import settings failed to load.';
		} finally {
			loading = false;
		}
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			await updateImportSettings({
				ebook_import_mode: ebookImportMode,
				ebook_passthrough_root: ebookPassthroughRoot,
				ebook_naming_template: ebookNamingTemplate,
				audiobook_layout_preset: audiobookLayoutPreset,
				calibre_command: calibreCommand
			});
			success = 'Import settings saved.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Import settings failed to save.';
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
			<Library class="h-4 w-4" />
			<span>Import</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Import and naming</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			Choose whether Athena manages ebook paths directly or hands them to a Calibre inbox, then pick
			the audiobook folder layout Athena should build on disk.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading import settings…</span>
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
					<span class="text-sm font-medium text-stone-700">Ebook import mode</span>
					<select bind:value={ebookImportMode} class="input-shell">
						<option value="managed">Managed naming</option>
						<option value="passthrough">Passthrough inbox</option>
					</select>
				</label>
				{#if ebookImportMode === 'managed'}
					<label class="space-y-2">
						<span class="text-sm font-medium text-stone-700">Ebook naming template</span>
						<input
							bind:value={ebookNamingTemplate}
							class="input-shell"
							placeholder={'{author}/{title}/{title}'}
						/>
					</label>
				{:else}
					<label class="space-y-2">
						<span class="text-sm font-medium text-stone-700">Ebook passthrough directory</span>
						<input
							bind:value={ebookPassthroughRoot}
							class="input-shell"
							placeholder="/calibre/inbox"
						/>
					</label>
				{/if}
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Audiobook folder layout</span>
					<select bind:value={audiobookLayoutPreset} class="input-shell">
						<option value="author_title">Author / Title</option>
						<option value="title">Title only</option>
					</select>
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Calibre command</span>
					<input bind:value={calibreCommand} class="input-shell" placeholder="calibredb" />
				</label>
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

			<button class="action-button" disabled={saving} type="submit">
				<Save class="h-4 w-4" />
				<span>{saving ? 'Saving…' : 'Save import settings'}</span>
			</button>
		</form>
	{/if}
</div>
