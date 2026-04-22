<script lang="ts">
	import { onMount } from 'svelte';
	import { getNotificationSettings, updateNotificationSettings } from '$lib/api';
	import type { NotificationTargetKind } from '$lib/types/NotificationTargetKind';

	let enabled = $state(false);
	let targetKind = $state('webhook' as NotificationTargetKind);
	let targetUrl = $state('');
	let authHeader = $state('');
	let authToken = $state('');
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');
	let hasAuthToken = $state(false);

	async function loadSettings() {
		loading = true;
		error = '';
		try {
			const settings = await getNotificationSettings();
			enabled = settings.enabled;
			targetKind = settings.target_kind;
			targetUrl = settings.target_url;
			authHeader = settings.auth_header ?? '';
			hasAuthToken = settings.has_auth_token;
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'Notification settings failed to load.';
		} finally {
			loading = false;
		}
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		error = '';
		success = '';
		try {
			const settings = await updateNotificationSettings({
				enabled,
				target_kind: targetKind,
				target_url: targetUrl,
				auth_header: authHeader || null,
				auth_token: authToken || null,
				clear_auth_token: false
			});
			hasAuthToken = settings.has_auth_token;
			authToken = '';
			success = 'Notification settings saved.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Notification settings failed to save.';
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
		<p class="eyebrow">Notifications</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Webhook and ntfy delivery</h1>
	</div>

	<form class="dashboard-card space-y-5" onsubmit={submit}>
		{#if loading}
			<p class="text-sm text-stone-600">Loading notification settings…</p>
		{:else}
			<label class="flex items-center gap-3 text-sm text-stone-700">
				<input bind:checked={enabled} class="h-4 w-4 accent-teal-900" type="checkbox" />
				<span>Enable notifications</span>
			</label>

			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Target kind</span>
				<select bind:value={targetKind} class="input-shell">
					<option value="webhook">Webhook</option>
					<option value="ntfy">ntfy</option>
				</select>
			</label>

			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Target URL</span>
				<input bind:value={targetUrl} class="input-shell" placeholder="https://notify.example.com" />
			</label>

			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Auth header</span>
				<input bind:value={authHeader} class="input-shell" placeholder="Authorization" />
			</label>

			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Auth token</span>
				<input bind:value={authToken} class="input-shell" placeholder={hasAuthToken ? 'Stored token will be kept unless replaced' : 'Optional token'} />
			</label>

			{#if error}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">{error}</div>
			{/if}
			{#if success}
				<div class="rounded-[1.4rem] border border-teal-200 bg-teal-50 px-4 py-4 text-sm text-teal-900">{success}</div>
			{/if}

			<button class="action-button" disabled={saving} type="submit">
				<span>{saving ? 'Saving…' : 'Save notifications'}</span>
			</button>
		{/if}
	</form>
</div>
