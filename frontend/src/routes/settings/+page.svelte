<script lang="ts">
	import { onMount } from 'svelte';
	import {
		ArrowRight,
		Cable,
		Database,
		Download,
		Headphones,
		FolderTree,
		LoaderCircle,
		ShieldCheck
	} from 'lucide-svelte';
	import { getRuntimeSettings, listSyncedIndexers } from '$lib/api';
	import type { RuntimeSettingsRecord } from '$lib/types/RuntimeSettingsRecord';

	let runtime = $state(null as RuntimeSettingsRecord | null);
	let syncedCount = $state(0);
	let loading = $state(true);
	let error = $state('');

	function qbStatus(): string {
		if (!runtime) return 'Loading';
		const qb = runtime.download_clients.qbittorrent;
		if (!qb.enabled) return 'Disabled';
		if (qb.base_url && qb.username && qb.has_password) return 'Configured';
		return 'Incomplete';
	}

	function prowlarrStatus(): string {
		if (!runtime) return 'Loading';
		const prowlarr = runtime.integrations.prowlarr;
		if (!prowlarr.enabled && !prowlarr.sync_enabled) return 'Disabled';
		if (prowlarr.base_url && prowlarr.has_api_key) return 'Configured';
		return 'Incomplete';
	}

	function audiobookshelfStatus(): string {
		if (!runtime) return 'Loading';
		const audiobookshelf = runtime.integrations.audiobookshelf;
		if (!audiobookshelf) return 'Disabled';
		if (!audiobookshelf.enabled) return 'Disabled';
		if (audiobookshelf.base_url && audiobookshelf.library_id && audiobookshelf.has_api_key) {
			return 'Configured';
		}
		return 'Incomplete';
	}

	async function loadOverview() {
		loading = true;
		error = '';

		try {
			const [settings, syncedIndexers] = await Promise.all([
				getRuntimeSettings(),
				listSyncedIndexers()
			]);
			runtime = settings;
			syncedCount = syncedIndexers.length;
		} catch (loadError) {
			error =
				loadError instanceof Error
					? loadError.message
					: 'The settings center could not load.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadOverview();
	});

	const cards = [
		{
			href: '/settings/download-clients/qbittorrent',
			title: 'qBittorrent',
			body: 'Cookie-backed WebUI settings, category routing, and connection testing.',
			icon: Download
		},
		{
			href: '/settings/integrations/prowlarr',
			title: 'Prowlarr',
			body: 'Application sync settings and connection testing for the Readarr-compatible surface.',
			icon: Cable
		},
		{
			href: '/settings/integrations/audiobookshelf',
			title: 'Audiobookshelf',
			body: 'Audiobook library scan settings used after import completion.',
			icon: Headphones
		},
		{
			href: '/settings/storage',
			title: 'Storage',
			body: 'Absolute media roots that feed import and sync workflows.',
			icon: Database
		},
		{
			href: '/settings/synced-indexers',
			title: 'Synced indexers',
			body: 'Read-only visibility into the indexers Prowlarr has pushed into Athena.',
			icon: FolderTree
		}
	];
</script>

<div class="space-y-6">
	<div class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(15,23,42,0.98),rgba(12,74,110,0.92))] text-stone-50">
		<div class="space-y-4">
			<p class="eyebrow border-white/20 bg-white/10 text-stone-50">Settings overview</p>
			<h1 class="font-serif text-4xl tracking-tight">Admin settings center</h1>
			<p class="max-w-3xl text-sm leading-6 text-stone-200 sm:text-base">
				Persisted runtime settings now own the operational surface. Changes save to SQLite and
				API consumers read the current values instead of stale environment defaults.
			</p>
		</div>
	</div>

	{#if loading}
		<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading runtime settings…</span>
		</div>
	{:else if error}
		<div class="dashboard-card border border-red-200 bg-red-50 text-sm text-red-700">{error}</div>
	{:else if runtime}
		<div class="space-y-4">
			<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
				<div class="dashboard-card">
					<p class="eyebrow mb-4">
						<Download class="h-4 w-4" />
						<span>Download client</span>
					</p>
					<p class="font-serif text-3xl text-stone-950">{qbStatus()}</p>
					<p class="mt-2 text-sm text-stone-600">
						{runtime.download_clients.qbittorrent.base_url || 'No qBittorrent URL saved yet.'}
					</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4 bg-amber-500/12 text-amber-900">
						<Cable class="h-4 w-4" />
						<span>Prowlarr</span>
					</p>
					<p class="font-serif text-3xl text-stone-950">{prowlarrStatus()}</p>
					<p class="mt-2 text-sm text-stone-600">
						{runtime.integrations.prowlarr.base_url || 'No Prowlarr URL saved yet.'}
					</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4 bg-teal-900/12 text-teal-950">
						<Database class="h-4 w-4" />
						<span>Storage</span>
					</p>
					<p class="font-serif text-3xl text-stone-950">{runtime.storage.ebooks_root}</p>
					<p class="mt-2 text-sm text-stone-600">Audiobooks root: {runtime.storage.audiobooks_root}</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4 bg-sky-500/12 text-sky-900">
						<Headphones class="h-4 w-4" />
						<span>Audiobookshelf</span>
					</p>
					<p class="font-serif text-3xl text-stone-950">{audiobookshelfStatus()}</p>
					<p class="mt-2 text-sm text-stone-600">
						{runtime.integrations.audiobookshelf?.base_url || 'No Audiobookshelf URL saved yet.'}
					</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4 bg-stone-900/8 text-stone-900">
						<ShieldCheck class="h-4 w-4" />
						<span>Synced indexers</span>
					</p>
					<p class="font-serif text-3xl text-stone-950">{syncedCount}</p>
					<p class="mt-2 text-sm text-stone-600">Prowlarr-managed indexers currently stored in Athena.</p>
				</div>
			</div>

			<div class="grid gap-4 xl:grid-cols-3">
				{#each cards as card}
					<a class="dashboard-card group transition hover:-translate-y-0.5 hover:border-teal-900/25" href={card.href}>
						<div class="flex items-start justify-between gap-4">
							<div class="space-y-3">
								<p class="eyebrow">
									<card.icon class="h-4 w-4" />
									<span>{card.title}</span>
								</p>
								<div>
									<h2 class="font-serif text-2xl text-stone-950">{card.title}</h2>
									<p class="mt-2 text-sm leading-6 text-stone-600">{card.body}</p>
								</div>
							</div>
							<ArrowRight class="mt-1 h-5 w-5 text-stone-400 transition group-hover:translate-x-1 group-hover:text-teal-900" />
						</div>
					</a>
				{/each}
			</div>
		</div>
	{/if}
</div>
