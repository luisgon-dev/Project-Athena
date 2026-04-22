<script lang="ts">
	import { onMount } from 'svelte';
	import { createUser, listUsers, updateUser } from '$lib/api';
	import type { CreateUserRequest } from '$lib/types/CreateUserRequest';
	import type { UserRecord } from '$lib/types/UserRecord';
	import type { UserRole } from '$lib/types/UserRole';

	let users = $state([] as UserRecord[]);
	let loading = $state(true);
	let error = $state('');
	let createError = $state('');
	let username = $state('');
	let password = $state('');
	let role = $state('requester' as UserRole);

	async function loadUsers() {
		loading = true;
		error = '';
		try {
			users = await listUsers();
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Users failed to load.';
		} finally {
			loading = false;
		}
	}

	async function submitCreate(event: SubmitEvent) {
		event.preventDefault();
		createError = '';
		try {
			await createUser({ username, password, role } satisfies CreateUserRequest);
			username = '';
			password = '';
			role = 'requester';
			await loadUsers();
		} catch (submitError) {
			createError = submitError instanceof Error ? submitError.message : 'Failed to create user.';
		}
	}

	async function toggleDisabled(user: UserRecord) {
		await updateUser(user.id, { disabled: !user.disabled, role: null, password: null });
		await loadUsers();
	}

	async function promote(user: UserRecord, nextRole: UserRole) {
		await updateUser(user.id, { role: nextRole, disabled: null, password: null });
		await loadUsers();
	}

	onMount(() => {
		void loadUsers();
	});
</script>

<div class="space-y-6">
	<div class="dashboard-card">
		<p class="eyebrow">Local auth</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Users and roles</h1>
	</div>

	<div class="grid gap-6 xl:grid-cols-[minmax(18rem,0.8fr)_minmax(0,1.2fr)]">
		<form class="dashboard-card space-y-4" onsubmit={submitCreate}>
			<div>
				<h2 class="font-serif text-2xl text-stone-950">Create user</h2>
			</div>
			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Username</span>
				<input bind:value={username} class="input-shell" />
			</label>
			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Password</span>
				<input bind:value={password} class="input-shell" type="password" />
			</label>
			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Role</span>
				<select bind:value={role} class="input-shell">
					<option value="requester">Requester</option>
					<option value="trusted">Trusted</option>
					<option value="admin">Admin</option>
				</select>
			</label>
			{#if createError}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">{createError}</div>
			{/if}
			<button class="action-button w-full justify-center" type="submit">Create user</button>
		</form>

		<div class="dashboard-card space-y-4">
			<h2 class="font-serif text-2xl text-stone-950">Existing users</h2>
			{#if loading}
				<p class="text-sm text-stone-600">Loading users…</p>
			{:else if error}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">{error}</div>
			{:else}
				<div class="grid gap-3">
					{#each users as user}
						<div class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4">
							<div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
								<div>
									<p class="font-semibold text-stone-900">{user.username}</p>
									<p class="text-sm text-stone-600">{user.role}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<button class="ghost-button" onclick={() => promote(user, 'requester')} type="button">Requester</button>
									<button class="ghost-button" onclick={() => promote(user, 'trusted')} type="button">Trusted</button>
									<button class="ghost-button" onclick={() => promote(user, 'admin')} type="button">Admin</button>
									<button class="ghost-button" onclick={() => toggleDisabled(user)} type="button">
										{user.disabled ? 'Enable' : 'Disable'}
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
