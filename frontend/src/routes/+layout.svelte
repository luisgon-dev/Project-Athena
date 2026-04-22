<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		Compass,
		LibraryBig,
		LogOut,
		Radar,
		ScanLine,
		Shield,
		SlidersHorizontal,
		Users
	} from 'lucide-svelte';
	import { authState, logoutAndReset, refreshAuth } from '$lib/auth';

	let { children } = $props();

	async function syncAuth() {
		try {
			const state = await refreshAuth();
			const pathname = page.url.pathname;
			if (state.setupRequired && pathname !== '/setup') {
				await goto('/setup');
				return;
			}
			if (!state.setupRequired && !state.user && pathname !== '/login') {
				await goto('/login');
			}
		} catch {
			authState.set({ loading: false, setupRequired: false, user: null });
		}
	}

	async function signOut() {
		await logoutAndReset();
		await goto('/login');
	}

	onMount(() => {
		void syncAuth();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Project Athena</title>
</svelte:head>

<div class="min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(15,118,110,0.18),_transparent_32%),radial-gradient(circle_at_top_right,_rgba(217,119,6,0.16),_transparent_26%),linear-gradient(180deg,_#f7f5ef_0%,_#f1ede2_100%)] text-stone-900">
	<div class="mx-auto flex min-h-screen max-w-7xl flex-col gap-6 px-4 py-5 sm:px-6 lg:px-8">
		<header class="glass-panel flex flex-col gap-6 px-5 py-5 sm:px-7">
			<div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
				<div class="space-y-3">
					<div class="eyebrow">
						<Compass class="h-4 w-4" />
						<span>Project Athena</span>
					</div>
					<div class="space-y-2">
						<h1 class="font-serif text-3xl font-semibold tracking-tight text-stone-950 sm:text-4xl">
							Operational calm for the request pipeline.
						</h1>
						<p class="max-w-3xl text-sm leading-6 text-stone-700 sm:text-base">
							Track every intake, search Open Library metadata, and inspect the event trail without
							leaving the shell that ships with the backend.
						</p>
					</div>
				</div>
				<div class="flex items-center gap-3 rounded-2xl border border-white/50 bg-white/65 px-4 py-3 shadow-sm backdrop-blur">
					<div class="rounded-2xl bg-teal-900 p-3 text-stone-50 shadow-lg">
						<LibraryBig class="h-6 w-6" />
					</div>
					<div>
						<p class="text-xs uppercase tracking-[0.24em] text-stone-500">Frontend modernization</p>
						<p class="font-serif text-lg text-stone-900">Phase 2 console</p>
					</div>
				</div>
			</div>
			<nav class="flex flex-wrap items-center gap-3 text-sm font-medium text-stone-700">
				<a class="nav-link" href="/requests/new">
					<Compass class="h-4 w-4" />
					<span>Request Books</span>
				</a>
				{#if $authState.user}
					<a class="nav-link" href="/my-requests">
						<Radar class="h-4 w-4" />
						<span>My Requests</span>
					</a>
					{#if $authState.user.role === 'admin'}
						<a class="nav-link" href="/">
							<Shield class="h-4 w-4" />
							<span>Admin Queue</span>
						</a>
						<a class="nav-link" href="/library/scan">
							<ScanLine class="h-4 w-4" />
							<span>Library Scan</span>
						</a>
						<a class="nav-link" href="/settings">
							<SlidersHorizontal class="h-4 w-4" />
							<span>Settings</span>
						</a>
						<a class="nav-link" href="/users">
							<Users class="h-4 w-4" />
							<span>Users</span>
						</a>
					{/if}
					<button class="nav-link" onclick={signOut} type="button">
						<LogOut class="h-4 w-4" />
						<span>Logout</span>
					</button>
				{/if}
			</nav>
		</header>

		<main class="flex-1">
			{@render children()}
		</main>
	</div>
</div>
