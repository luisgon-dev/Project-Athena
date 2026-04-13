<script lang="ts">
	import { onMount } from 'svelte';
	import { LoaderCircle, Rows3, Save } from 'lucide-svelte';
	import { getAcquisitionSettings, updateAcquisitionSettings } from '$lib/api';

	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');

	let minimumScore = $state('0.7');
	let autoAcquireScore = $state('0.93');
	let preferredLanguage = $state('');
	let blockedTerms = $state('');

	async function loadSettings() {
		loading = true;
		error = '';

		try {
			const settings = await getAcquisitionSettings();
			minimumScore = settings.minimum_score.toString();
			autoAcquireScore = settings.auto_acquire_score.toString();
			preferredLanguage = settings.preferred_language ?? '';
			blockedTerms = settings.blocked_terms.join(', ');
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'Acquisition settings failed to load.';
		} finally {
			loading = false;
		}
	}

	async function saveSettings() {
		saving = true;
		error = '';
		success = '';

		try {
			await updateAcquisitionSettings({
				minimum_score: Number(minimumScore),
				auto_acquire_score: Number(autoAcquireScore),
				preferred_language: preferredLanguage,
				blocked_terms: blockedTerms
					.split(',')
					.map((value) => value.trim())
					.filter(Boolean)
			});
			success = 'Acquisition settings saved.';
		} catch (saveError) {
			error =
				saveError instanceof Error ? saveError.message : 'Acquisition settings failed to save.';
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
			<Rows3 class="h-4 w-4" />
			<span>Acquisition</span>
		</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Matcher thresholds</h1>
		<p class="mt-2 max-w-3xl text-sm leading-6 text-stone-600">
			Set the minimum score that survives review, the auto-acquire threshold, and any blocked
			terms Athena should avoid when it eventually starts using synced indexers directly.
		</p>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading acquisition settings…</span>
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
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Minimum score</span>
					<input bind:value={minimumScore} class="input-shell" step="0.01" type="number" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Auto-acquire score</span>
					<input bind:value={autoAcquireScore} class="input-shell" step="0.01" type="number" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Preferred language</span>
					<input bind:value={preferredLanguage} class="input-shell" placeholder="en" />
				</label>
				<label class="space-y-2">
					<span class="text-sm font-medium text-stone-700">Blocked terms</span>
					<input bind:value={blockedTerms} class="input-shell" placeholder="abridged, dramatized" />
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
				<span>{saving ? 'Saving…' : 'Save acquisition settings'}</span>
			</button>
		</form>
	{/if}
</div>
