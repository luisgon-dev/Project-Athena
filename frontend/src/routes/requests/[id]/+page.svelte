<script lang="ts">
	import { onMount } from 'svelte';
	import {
		ArrowLeft,
		BadgeCheck,
		Clock3,
		LoaderCircle,
		NotebookTabs,
		Orbit,
		RefreshCw,
		ScanSearch,
		XCircle
	} from 'lucide-svelte';
	import {
		approveReviewCandidate,
		getRequestDetail,
		rejectReviewCandidate,
		retryRequestSearch
	} from '$lib/api';
	import type { RequestDetailRecord } from '$lib/types/RequestDetailRecord';
	import type { RequestEventRecord } from '$lib/types/RequestEventRecord';
	import type { ReviewQueueEntry } from '$lib/types/ReviewQueueEntry';

	let detail = $state(null as RequestDetailRecord | null);
	let loading = $state(true);
	let error = $state('');
	let actionError = $state('');
	let actionSuccess = $state('');
	let retrying = $state(false);
	let actingCandidateId = $state(null as bigint | number | null);
	let actingAction = $state('');

	function requestId(): string {
		return window.location.pathname.split('/').filter(Boolean).at(-1) ?? '';
	}

	function isPending(candidateId: bigint | number, action: string): boolean {
		return actingCandidateId === candidateId && actingAction === action;
	}

	function eventSummary(event: RequestEventRecord): string {
		try {
			const payload = JSON.parse(event.payload_json) as {
				title?: string;
				author?: string;
				media_type?: string;
				outcome?: string;
				queued_candidates?: number;
				candidate_title?: string;
				candidate_source?: string;
				rejected_candidate_id?: string;
				qualified_candidates?: number;
				top_score?: number;
			};
			if (event.kind === 'Created') {
				return `${payload.title ?? 'Request'} by ${payload.author ?? 'Unknown author'} (${payload.media_type ?? 'unknown media'})`;
			}
			if (event.kind === 'SearchCompleted') {
				return `${payload.outcome ?? 'search'} with ${payload.qualified_candidates ?? 0} qualified candidates; top score ${payload.top_score ?? 0}`;
			}
			if (event.kind === 'ReviewQueued') {
				return `${payload.queued_candidates ?? 0} candidates moved into manual review.`;
			}
			if (event.kind === 'ReviewApproved') {
				return `${payload.candidate_title ?? 'Candidate'} from ${payload.candidate_source ?? 'unknown source'} was approved for dispatch.`;
			}
			if (event.kind === 'ReviewRejected') {
				return `Candidate ${payload.rejected_candidate_id ?? 'unknown'} was rejected and excluded from the request.`;
			}
		} catch {
			// Fall through to the generic event summary.
		}

		return event.kind;
	}

	function scoreLabel(score: number): string {
		return score.toFixed(2);
	}

	function candidateMetadataSummary(candidate: ReviewQueueEntry): string[] {
		const parts = [] as string[];
		if (candidate.candidate.narrator) {
			parts.push(`Narrator: ${candidate.candidate.narrator}`);
		}
		if (candidate.candidate.detected_language) {
			parts.push(`Language: ${candidate.candidate.detected_language}`);
		}
		parts.push(`Graphic audio: ${candidate.candidate.graphic_audio ? 'yes' : 'no'}`);
		return parts;
	}

	async function runRefreshableAction(
		task: () => Promise<RequestDetailRecord>,
		successMessage: string,
		options?: { candidateId?: bigint | number; action?: string }
	) {
		actionError = '';
		actionSuccess = '';
		actingCandidateId = options?.candidateId ?? null;
		actingAction = options?.action ?? '';

		try {
			detail = await task();
			actionSuccess = successMessage;
		} catch (taskError) {
			actionError = taskError instanceof Error ? taskError.message : 'The request action failed.';
		} finally {
			actingCandidateId = null;
			actingAction = '';
		}
	}

	async function approveCandidate(candidate: ReviewQueueEntry) {
		await runRefreshableAction(
			() => approveReviewCandidate(requestId(), candidate.id),
			'Candidate approved and dispatched to qBittorrent.',
			{ candidateId: candidate.id, action: 'approve' }
		);
	}

	async function rejectCandidate(candidate: ReviewQueueEntry) {
		await runRefreshableAction(
			() => rejectReviewCandidate(requestId(), candidate.id),
			'Candidate rejected. Athena refreshed the queue state.',
			{ candidateId: candidate.id, action: 'reject' }
		);
	}

	async function retrySearch() {
		retrying = true;
		actionError = '';
		actionSuccess = '';

		try {
			detail = await retryRequestSearch(requestId());
			actionSuccess = 'Search retried with the current acquisition settings.';
		} catch (retryError) {
			actionError = retryError instanceof Error ? retryError.message : 'The search retry failed.';
		} finally {
			retrying = false;
		}
	}

	async function loadDetail() {
		loading = true;
		error = '';

		try {
			detail = await getRequestDetail(requestId());
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
		<section
			class="dashboard-card rounded-[1.6rem] border border-red-200 bg-red-50 text-sm text-red-700"
		>
			{error}
		</section>
	{:else if detail}
		<div class="grid gap-6 xl:grid-cols-[minmax(0,1.35fr)_minmax(19rem,0.85fr)]">
			<section class="space-y-6">
				<div
					class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(12,74,110,0.96),rgba(15,118,110,0.84))] text-stone-50"
				>
					<div class="space-y-4">
						<div class="flex flex-wrap gap-2">
							<span class="status-pill border-white/20 bg-white/10 text-stone-50"
								>{detail.request.state}</span
							>
							<span class="status-pill border-white/20 bg-white/10 text-stone-50"
								>{detail.request.media_type}</span
							>
						</div>
						<div class="flex flex-wrap items-start justify-between gap-4">
							<div class="space-y-3">
								<h1 class="font-serif text-4xl font-semibold tracking-tight">
									{detail.request.title}
								</h1>
								<p class="text-base text-stone-200">{detail.request.author}</p>
							</div>
							<div class="min-w-[12rem] rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs tracking-[0.2em] text-stone-300 uppercase">Review queue</p>
								<p class="mt-2 font-serif text-3xl text-stone-50">{detail.review_queue.length}</p>
								<p class="mt-1 text-sm text-stone-200">Manual candidates currently waiting.</p>
							</div>
						</div>
						<div class="grid gap-4 sm:grid-cols-3">
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs tracking-[0.2em] text-stone-300 uppercase">Canonical work</p>
								<p class="mt-2 font-medium text-stone-50">
									{detail.request.external_work_id || 'Unresolved'}
								</p>
							</div>
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs tracking-[0.2em] text-stone-300 uppercase">Preferred language</p>
								<p class="mt-2 font-medium text-stone-50">
									{detail.request.preferred_language ?? 'Any'}
								</p>
							</div>
							<div class="rounded-[1.35rem] border border-white/15 bg-white/10 p-4">
								<p class="text-xs tracking-[0.2em] text-stone-300 uppercase">Created</p>
								<p class="mt-2 font-medium text-stone-50">{detail.request.created_at}</p>
							</div>
						</div>
					</div>
				</div>

				<div class="dashboard-card space-y-4">
					<div class="flex flex-wrap items-start justify-between gap-4">
						<div>
							<p class="eyebrow">
								<ScanSearch class="h-4 w-4" />
								<span>Review queue</span>
							</p>
							<h2 class="mt-3 font-serif text-2xl text-stone-950">Candidate review</h2>
							<p class="mt-2 text-sm leading-6 text-stone-600">
								Each candidate expands in place so the page stays compact while still exposing score
								reasons, source metadata, and approval controls.
							</p>
						</div>
						<span class="status-pill bg-stone-900/8 text-stone-900">
							{detail.review_queue.length} queued
						</span>
					</div>

					{#if detail.review_queue.length === 0}
						<div class="rounded-[1.45rem] bg-stone-100/90 px-4 py-4 text-sm text-stone-600">
							No candidates are waiting for manual review on this request.
						</div>
					{:else}
						<div class="space-y-4">
							{#each detail.review_queue as candidate}
								<details
									class="rounded-[1.5rem] border border-stone-200 bg-stone-50/95 p-5 open:shadow-sm"
								>
									<summary class="cursor-pointer list-none">
										<div class="flex flex-wrap items-start justify-between gap-4">
											<div class="space-y-2">
												<div class="flex flex-wrap items-center gap-2">
													<span class="status-pill bg-teal-900/8 text-teal-950">
														Score {scoreLabel(candidate.score)}
													</span>
													<span class="status-pill bg-stone-900/8 text-stone-900">
														{candidate.candidate.source}
													</span>
													<span class="status-pill bg-stone-900/8 text-stone-900">
														{candidate.candidate.indexer}
													</span>
												</div>
												<h3 class="font-serif text-2xl text-stone-950">
													{candidate.candidate.title}
												</h3>
												<p class="text-sm text-stone-600">
													Protocol: {candidate.candidate.protocol} · Size: {String(
														candidate.candidate.size_bytes
													)} bytes
												</p>
											</div>
											<span class="text-xs tracking-[0.18em] text-stone-400 uppercase">Expand</span>
										</div>
									</summary>

									<div class="mt-5 space-y-4 border-t border-stone-200 pt-4">
										{#if candidate.explanation.length > 0}
											<ul class="grid gap-2 text-sm text-stone-600">
												{#each candidate.explanation as explanation}
													<li class="rounded-[1.2rem] bg-white px-3 py-2">{explanation}</li>
												{/each}
											</ul>
										{/if}

										<ul class="grid gap-2 text-sm text-stone-600">
											{#each candidateMetadataSummary(candidate) as metadataLine}
												<li class="rounded-[1.2rem] bg-white px-3 py-2">{metadataLine}</li>
											{/each}
										</ul>

										{#if candidate.candidate.download_url}
											<p class="text-sm break-all text-stone-600">
												Download URL:
												<a
													class="text-teal-900 underline decoration-teal-900/30 underline-offset-4"
													href={candidate.candidate.download_url}
													rel="noreferrer"
													target="_blank"
												>
													{candidate.candidate.download_url}
												</a>
											</p>
										{/if}

										<div class="flex flex-wrap gap-2">
											<button
												class="action-button"
												disabled={actingCandidateId !== null}
												onclick={() => {
													void approveCandidate(candidate);
												}}
												type="button"
											>
												<BadgeCheck class="h-4 w-4" />
												<span>{isPending(candidate.id, 'approve') ? 'Approving…' : 'Approve'}</span>
											</button>
											<button
												class="ghost-button"
												disabled={actingCandidateId !== null}
												onclick={() => {
													void rejectCandidate(candidate);
												}}
												type="button"
											>
												<XCircle class="h-4 w-4" />
												<span>{isPending(candidate.id, 'reject') ? 'Rejecting…' : 'Reject'}</span>
											</button>
										</div>
									</div>
								</details>
							{/each}
						</div>
					{/if}
				</div>
			</section>

			<aside class="space-y-6 xl:sticky xl:top-6 xl:self-start">
				<div class="dashboard-card space-y-4">
					<div class="flex flex-wrap items-start justify-between gap-4">
						<div>
							<p class="eyebrow">
								<BadgeCheck class="h-4 w-4" />
								<span>Fulfillment controls</span>
							</p>
							<h2 class="mt-3 font-serif text-2xl text-stone-950">Operator actions</h2>
							<p class="mt-2 text-sm leading-6 text-stone-600">
								Retry search here after adjusting thresholds, or act directly from the review
								candidate cards.
							</p>
						</div>
						{#if detail.request.state === 'no_match' || detail.request.state === 'review'}
							<button class="ghost-button" disabled={retrying} onclick={retrySearch} type="button">
								<RefreshCw class={`h-4 w-4 ${retrying ? 'animate-spin' : ''}`} />
								<span>{retrying ? 'Retrying…' : 'Retry search'}</span>
							</button>
						{/if}
					</div>

					{#if actionError}
						<div
							class="rounded-[1.35rem] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
						>
							{actionError}
						</div>
					{/if}
					{#if actionSuccess}
						<div
							class="rounded-[1.35rem] border border-teal-200 bg-teal-50 px-4 py-3 text-sm text-teal-900"
						>
							{actionSuccess}
						</div>
					{/if}
				</div>

				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">
							<Orbit class="h-4 w-4" />
							<span>Manifestation preferences</span>
						</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Fulfillment cues</h2>
					</div>
					<div class="grid gap-3">
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs tracking-[0.2em] text-stone-500 uppercase">Edition title</p>
							<p class="mt-2 text-sm text-stone-800">
								{detail.request.manifestation.edition_title ?? 'Any'}
							</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs tracking-[0.2em] text-stone-500 uppercase">Preferred narrator</p>
							<p class="mt-2 text-sm text-stone-800">
								{detail.request.manifestation.preferred_narrator ?? 'Any'}
							</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs tracking-[0.2em] text-stone-500 uppercase">Preferred publisher</p>
							<p class="mt-2 text-sm text-stone-800">
								{detail.request.manifestation.preferred_publisher ?? 'Any'}
							</p>
						</div>
						<div class="rounded-[1.45rem] bg-stone-100/90 p-4">
							<p class="text-xs tracking-[0.2em] text-stone-500 uppercase">Graphic audio</p>
							<p class="mt-2 text-sm text-stone-800">
								{detail.request.manifestation.graphic_audio ? 'Requested' : 'Not requested'}
							</p>
						</div>
					</div>
				</div>

				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">
							<NotebookTabs class="h-4 w-4" />
							<span>Event timeline</span>
						</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Recorded history</h2>
					</div>
					<div class="max-h-[32rem] space-y-3 overflow-y-auto pr-1">
						{#each detail.events as event}
							<div class="rounded-[1.45rem] border border-stone-200 bg-stone-50/90 px-4 py-4">
								<div class="flex items-start justify-between gap-4">
									<div>
										<p class="font-semibold text-stone-900">{event.kind}</p>
										<p class="mt-1 text-sm leading-6 text-stone-600">{eventSummary(event)}</p>
									</div>
									<div
										class="flex items-center gap-2 text-xs tracking-[0.18em] text-stone-400 uppercase"
									>
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
