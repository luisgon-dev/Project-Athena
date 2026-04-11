<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Activity,
		ArrowRight,
		CircleDot,
		Clock3,
		LibraryBig,
		RefreshCw,
		Sparkles
	} from 'lucide-svelte';
	import { listRequests } from '$lib/api';
	import type { RequestListRecord } from '$lib/types/RequestListRecord';

	let requests = $state([] as RequestListRecord[]);
	let loading = $state(true);
	let error = $state('');

	const activeCount = $derived(requests.length);
	const requestedCount = $derived(requests.filter((request) => request.state === 'requested').length);
	const importedCount = $derived(requests.filter((request) => request.state === 'imported').length);

	function formatStamp(value: string): string {
		return value.replace('T', ' ').replace('.000', '');
	}

	function mediaLabel(value: RequestListRecord['media_type']): string {
		return value === 'Audiobook' ? 'Audiobook' : 'Ebook';
	}

	async function loadDashboard() {
		loading = true;
		error = '';

		try {
			requests = await listRequests();
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'The dashboard could not reach the request API.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadDashboard();
	});
</script>

<div class="grid gap-6 lg:grid-cols-[minmax(0,1.65fr)_minmax(18rem,0.9fr)]">
	<section class="space-y-6">
		<div class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(15,23,42,0.98),rgba(19,78,74,0.92))] text-stone-50">
			<div class="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
				<div class="space-y-4">
					<div class="eyebrow border-white/20 bg-white/10 text-stone-50">
						<Sparkles class="h-4 w-4" />
						<span>Live intake board</span>
					</div>
					<div class="space-y-3">
						<h1 class="font-serif text-4xl font-semibold tracking-tight text-stone-50">
							Request Radar
						</h1>
						<p class="max-w-2xl text-sm leading-6 text-stone-200 sm:text-base">
							Every request at a glance, with the freshest activity pinned above the noise.
							Use the metadata wizard for new intake and drop into a request detail page for the full
							event trail.
						</p>
					</div>
				</div>
				<div class="flex flex-wrap gap-3">
					<a class="action-button bg-amber-500 text-stone-950 hover:bg-amber-400" href="/requests/new">
						<LibraryBig class="h-4 w-4" />
						<span>New request</span>
					</a>
					<button class="ghost-button border-white/25 bg-white/10 text-stone-50 hover:border-white hover:bg-white hover:text-stone-950" onclick={loadDashboard} type="button">
						<RefreshCw class="h-4 w-4" />
						<span>Refresh</span>
					</button>
				</div>
			</div>
		</div>

		<div class="grid gap-4 md:grid-cols-3">
			<div class="dashboard-card">
				<p class="eyebrow mb-4">
					<Activity class="h-4 w-4" />
					<span>Pipeline</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{activeCount}</p>
				<p class="mt-2 text-sm text-stone-600">Tracked requests currently visible in the admin surface.</p>
			</div>
			<div class="dashboard-card">
				<p class="eyebrow mb-4 bg-amber-500/12 text-amber-900">
					<Clock3 class="h-4 w-4" />
					<span>Queued</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{requestedCount}</p>
				<p class="mt-2 text-sm text-stone-600">Requests still waiting on downstream fulfillment.</p>
			</div>
			<div class="dashboard-card">
				<p class="eyebrow mb-4 bg-teal-900/12 text-teal-950">
					<CircleDot class="h-4 w-4" />
					<span>Imported</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{importedCount}</p>
				<p class="mt-2 text-sm text-stone-600">Records that already crossed the import line.</p>
			</div>
		</div>

		<div class="dashboard-card space-y-5">
			<div class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
				<div>
					<p class="eyebrow">
						<Activity class="h-4 w-4" />
						<span>Recent requests</span>
					</p>
					<h2 class="mt-3 font-serif text-2xl text-stone-950">Operational feed</h2>
				</div>
				<p class="text-sm text-stone-500">Newest records are sorted to the top by the backend API.</p>
			</div>

			{#if loading}
				<div class="rounded-[1.5rem] border border-dashed border-stone-300 bg-stone-50/80 px-5 py-8 text-sm text-stone-600">
					Loading request activity from the Phase 2 API…
				</div>
			{:else if error}
				<div class="rounded-[1.5rem] border border-red-200 bg-red-50 px-5 py-6 text-sm text-red-700">
					{error}
				</div>
			{:else if requests.length === 0}
				<div class="rounded-[1.5rem] border border-dashed border-stone-300 bg-stone-50/80 px-5 py-8 text-sm text-stone-600">
					No requests exist yet. Start with the metadata wizard and the board will populate on refresh.
				</div>
			{:else}
				<div class="grid gap-3">
					{#each requests as request}
						<a
							class="group rounded-[1.5rem] border border-stone-200 bg-stone-50/90 px-5 py-5 transition hover:-translate-y-0.5 hover:border-teal-800/30 hover:bg-white"
							href={`/requests/${request.id}`}
						>
							<div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
								<div class="space-y-2">
									<div class="flex flex-wrap items-center gap-2">
										<span class="status-pill">{request.state}</span>
										<span class="status-pill bg-white">{mediaLabel(request.media_type)}</span>
									</div>
									<h3 class="font-serif text-2xl text-stone-950">{request.title}</h3>
									<p class="text-sm text-stone-600">{request.author}</p>
								</div>
								<div class="flex items-center gap-4 text-sm text-stone-500">
									<div class="text-right">
										<p class="font-medium uppercase tracking-[0.18em] text-stone-400">Created</p>
										<p class="mt-1 text-stone-700">{formatStamp(request.created_at)}</p>
									</div>
									<ArrowRight class="h-5 w-5 transition group-hover:translate-x-1 group-hover:text-teal-900" />
								</div>
							</div>
						</a>
					{/each}
				</div>
			{/if}
		</div>
	</section>

	<aside class="space-y-6">
		<div class="dashboard-card space-y-4">
			<p class="eyebrow">
				<Sparkles class="h-4 w-4" />
				<span>Operator notes</span>
			</p>
			<h2 class="font-serif text-2xl text-stone-950">How this surface works</h2>
			<ul class="space-y-3 text-sm leading-6 text-stone-600">
				<li>Search metadata first so new requests start from a canonical Open Library work.</li>
				<li>Each detail page includes the request event timeline from the Rust API.</li>
				<li>The backend owns sorting, status, and error envelopes; the SPA stays thin and typed.</li>
			</ul>
		</div>

		<div class="dashboard-card space-y-4">
			<p class="eyebrow bg-amber-500/12 text-amber-900">
				<Clock3 class="h-4 w-4" />
				<span>Phase targets</span>
			</p>
			<div class="space-y-3 text-sm text-stone-600">
				<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<p class="font-semibold text-stone-900">JSON API in front</p>
					<p class="mt-1">This dashboard reads directly from `/api/v1/requests` and refreshes in place.</p>
				</div>
				<div class="rounded-[1.35rem] bg-stone-100/90 px-4 py-4">
					<p class="font-semibold text-stone-900">Event-first detail pages</p>
					<p class="mt-1">Follow the trail from intake through import without opening server-rendered templates.</p>
				</div>
			</div>
		</div>
	</aside>
</div>
