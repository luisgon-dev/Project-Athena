<script lang="ts">
	import { goto } from '$app/navigation';
	import { setupAndRefresh } from '$lib/auth';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		loading = true;
		error = '';
		try {
			await setupAndRefresh({ username, password });
			await goto('/');
		} catch (submitError) {
			error = submitError instanceof Error ? submitError.message : 'Setup failed.';
		} finally {
			loading = false;
		}
	}
</script>

<div class="mx-auto max-w-xl">
	<form class="dashboard-card space-y-5" onsubmit={submit}>
		<div>
			<p class="eyebrow">First run</p>
			<h1 class="mt-3 font-serif text-3xl text-stone-950">Create the initial admin account</h1>
			<p class="mt-2 text-sm leading-6 text-stone-600">
				This local account owns the first-release admin console. Additional requester and trusted
				users are created from the Users screen after setup.
			</p>
		</div>

		<label class="space-y-2 text-sm text-stone-600">
			<span class="font-medium text-stone-800">Admin username</span>
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
			<span>{loading ? 'Creating admin…' : 'Create admin account'}</span>
		</button>
	</form>
</div>
