<script lang="ts">
	import { goto } from '$app/navigation';
	import { loginAndRefresh } from '$lib/auth';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		loading = true;
		error = '';
		try {
			await loginAndRefresh({ username, password });
			await goto('/');
		} catch (submitError) {
			error = submitError instanceof Error ? submitError.message : 'Login failed.';
		} finally {
			loading = false;
		}
	}
</script>

<div class="mx-auto max-w-xl">
	<form class="dashboard-card space-y-5" onsubmit={submit}>
		<div>
			<p class="eyebrow">Authentication</p>
			<h1 class="mt-3 font-serif text-3xl text-stone-950">Log in to Athena</h1>
		</div>

		<label class="space-y-2 text-sm text-stone-600">
			<span class="font-medium text-stone-800">Username</span>
			<input bind:value={username} class="input-shell" />
		</label>

		<label class="space-y-2 text-sm text-stone-600">
			<span class="font-medium text-stone-800">Password</span>
			<input bind:value={password} class="input-shell" type="password" />
		</label>

		{#if error}
			<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">{error}</div>
		{/if}

		<button class="action-button w-full justify-center" disabled={loading} type="submit">
			<span>{loading ? 'Signing in…' : 'Log in'}</span>
		</button>
	</form>
</div>
