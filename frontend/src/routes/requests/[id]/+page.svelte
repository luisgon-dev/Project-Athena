<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowLeft, Clock3, LoaderCircle, NotebookTabs, Orbit, ScanSearch } from 'lucide-svelte';
	import { getRequestDetail } from '$lib/api';
	import type { RequestDetailRecord } from '$lib/types/RequestDetailRecord';
	import type { RequestEventRecord } from '$lib/types/RequestEventRecord';

	let detail = $state(null as RequestDetailRecord | null);
	let loading = $state(true);
	let error = $state('');

	function eventSummary(event: RequestEventRecord): string {
		try {
			const payload = JSON.parse(event.payload_json) as {
				title?: string;
				author?: string;
				media_type?: string;
			};
			if (event.kind === 'Created') {
				return `${payload.title ?? 'Request'} by ${payload.author ?? 'Unknown author'} (${payload.media_type ?? 'unknown media'})`;
			}
		} catch {
			// Fall through to the generic event summary.
		}

		return event.kind;
	}

	async function loadDetail() {
		loading = true;
		error = '';

		const requestId = window.location.pathname.split('/').filter(Boolean).at(-1) ?? '';

		try {
			detail = await getRequestDetail(requestId);
		} catch (loadError) {
			detail = null;
			error = loadError instanceof Error ? loadError.message : 'Request detail failed to load.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadDetail();
	});
</script>

<div class="space-y-6">
	<div class="flex flex-wrap items-center justify-between gap-3">
		<a class="ghost-button" href="/">
			<ArrowLeft class="h-4 w-4" />
			<span>Back to dashboard</span>
		</a>
		<a class="action-button bg-teal-900 hover:bg-teal-800" href="/requests/new">
			<ScanSearch class="h-4 w-4" />
			<span>Open metadata wizard</span>
		</a>
	</div>

	{#if loading}
		<section class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
			<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
			<span>Loading request detail from the JSON API…</span>
		</section>
	{:else if error}
		<section class="dashboard-card rounded-[1.6rem] border border-red-200 bg-red-50 text-sm text-red-700">
			{error}
		</section>
	{:else if detail}
		<div class="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.95fr)]">
			<section class="space-y-6">
				<div class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(12,74,110,0.96),rgba(15,118,110,0.84))] text-stone-50">
					<div class="space-y-4">
						<div class="flex flex-wrap gap-2">
							<span class="status-pill border-white/20 bg-white/10 text-stone-50">{detail.request.state}</span>
							<span class="status-pill border-white/20 bg-white/10 text-stone-50">{detail.request.media_type}</span>
						</div>
						<div class="space-y-3">
							<h1 class="font-serif text-4xl font-semibold tracking-tight">{detail.request.title}</h1>
							<p class="text-base text-stone-200">{detail.request.author}</p>
						</div>
						<div class="grid gap-4 sm:grid-cols-3">
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs uppercase tracking-[0.2em] text-stone-300">Canonical work</p>
								<p class="mt-2 font-medium text-stone-50">{detail.request.external_work_id || 'Unresolved'}</p>
							</div>
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs uppercase tracking-[0.2em] text-stone-300">Preferred language</p>
								<p class="mt-2 font-medium text-stone-50">{detail.request.preferred_language ?? 'Any'}</p>
							</div>
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs uppercase tracking-[0.2em] text-stone-300">Created</p>
								<p class="mt-2 font-medium text-stone-50">{detail.request.created_at}</p>
							</div>
						</div>
					</div>
				</div>

				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">
							<Orbit class="h-4 w-4" />
							<span>Manifestation preferences</span>
						</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Fulfillment cues</h2>
					</div>
					<div class="grid gap-4 md:grid-cols-2">
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Edition title</p>
							<p class="mt-2 text-sm text-stone-800">{detail.request.manifestation.edition_title ?? 'Any'}</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Preferred narrator</p>
							<p class="mt-2 text-sm text-stone-800">{detail.request.manifestation.preferred_narrator ?? 'Any'}</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Preferred publisher</p>
							<p class="mt-2 text-sm text-stone-800">{detail.request.manifestation.preferred_publisher ?? 'Any'}</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs uppercase tracking-[0.2em] text-stone-500">Graphic audio</p>
							<p class="mt-2 text-sm text-stone-800">
								{detail.request.manifestation.graphic_audio ? 'Requested' : 'Not requested'}
							</p>
						</div>
					</div>
				</div>
			</section>

			<aside class="space-y-6">
				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">
							<NotebookTabs class="h-4 w-4" />
							<span>Event timeline</span>
						</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Recorded history</h2>
					</div>
					<div class="space-y-3">
						{#each detail.events as event}
							<div class="rounded-[1.45rem] border border-stone-200 bg-stone-50/90 px-4 py-4">
								<div class="flex items-start justify-between gap-4">
									<div>
										<p class="font-semibold text-stone-900">{event.kind}</p>
										<p class="mt-1 text-sm leading-6 text-stone-600">{eventSummary(event)}</p>
									</div>
									<div class="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-stone-400">
										<Clock3 class="h-4 w-4" />
										<span>{event.created_at}</span>
									</div>
								</div>
							</div>
						{/each}
					</div>
				</div>
			</aside>
		</div>
	{/if}
</div>
