<script lang="ts">
	import { Cable, Database, Download, LayoutList, Library, Rows3 } from 'lucide-svelte';

	let { children } = $props();

	const links = [
		{ href: '/settings', label: 'Overview', icon: LayoutList },
		{ href: '/settings/download-clients/qbittorrent', label: 'qBittorrent', icon: Download },
		{ href: '/settings/integrations/prowlarr', label: 'Prowlarr', icon: Cable },
		{ href: '/settings/storage', label: 'Storage', icon: Database },
		{ href: '/settings/import', label: 'Import', icon: Library },
		{ href: '/settings/acquisition', label: 'Acquisition', icon: Rows3 },
		{ href: '/settings/synced-indexers', label: 'Synced Indexers', icon: Download }
	];
</script>

<div class="grid gap-6 xl:grid-cols-[18rem_minmax(0,1fr)]">
	<aside class="dashboard-card h-fit space-y-4">
		<div>
			<p class="eyebrow">Settings center</p>
			<h2 class="mt-3 font-serif text-2xl text-stone-950">Operational controls</h2>
			<p class="mt-2 text-sm leading-6 text-stone-600">
				Download routing, storage, imports, and Prowlarr sync live here.
			</p>
		</div>
		<nav class="grid gap-2">
			{#each links as link}
				<a class="nav-link justify-start" href={link.href}>
					<link.icon class="h-4 w-4" />
					<span>{link.label}</span>
				</a>
			{/each}
		</nav>
	</aside>

	<div class="min-w-0">
		{@render children()}
	</div>
</div>
