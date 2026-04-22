<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { LoaderCircle, Search, ShieldCheck, ShieldX } from 'lucide-svelte';
	import { authState } from '$lib/auth';
	import {
		approveSubmission,
		getSubmissionDetail,
		listSubmissions,
		rejectSubmission,
		resolveSubmission,
		searchSubmissionCandidates
	} from '$lib/api';
	import type { RequestSubmissionDetailRecord } from '$lib/types/RequestSubmissionDetailRecord';
	import type { RequestSubmissionRecord } from '$lib/types/RequestSubmissionRecord';
	import type { SubmissionSearchCandidate } from '$lib/types/SubmissionSearchCandidate';

	let submissions = $state([] as RequestSubmissionRecord[]);
	let detail = $state(null as RequestSubmissionDetailRecord | null);
	let loading = $state(true);
	let error = $state('');
	let resolveQuery = $state('');
	let resolveResults = $state([] as SubmissionSearchCandidate[]);
	let resolving = $state(false);

	async function loadSubmissions() {
		loading = true;
		error = '';
		try {
			submissions = await listSubmissions($authState.user?.role === 'admin');
			const selectedId =
				page.url.searchParams.get('submission') ?? submissions[0]?.id ?? null;
			if (selectedId) {
				detail = await getSubmissionDetail(selectedId);
			} else {
				detail = null;
			}
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Failed to load submissions.';
		} finally {
			loading = false;
		}
	}

	async function openSubmission(submissionId: string) {
		detail = await getSubmissionDetail(submissionId);
	}

	async function runResolveSearch() {
		if (!detail) return;
		const response = await searchSubmissionCandidates({
			title: resolveQuery || detail.submission.title,
			author: detail.submission.author
		});
		resolveResults = response.works;
	}

	async function resolveWith(workId: string) {
		if (!detail) return;
		resolving = true;
		try {
			detail = await resolveSubmission(detail.submission.id, { selected_work_id: workId });
			await loadSubmissions();
		} finally {
			resolving = false;
		}
	}

	async function approveCurrent() {
		if (!detail) return;
		detail = await approveSubmission(detail.submission.id);
		await loadSubmissions();
	}

	async function rejectCurrent() {
		if (!detail) return;
		detail = await rejectSubmission(detail.submission.id);
		await loadSubmissions();
	}

	onMount(() => {
		void loadSubmissions();
	});
</script>

<div class="grid gap-6 xl:grid-cols-[minmax(18rem,0.9fr)_minmax(0,1.4fr)]">
	<div class="dashboard-card space-y-4">
		<div>
			<p class="eyebrow">Portal queue</p>
			<h1 class="mt-3 font-serif text-2xl text-stone-950">
				{$authState.user?.role === 'admin' ? 'Submission queue' : 'My requests'}
			</h1>
		</div>

		{#if loading}
			<div class="flex items-center gap-3 text-sm text-stone-600">
				<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
				<span>Loading submissions…</span>
			</div>
		{:else if error}
			<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">{error}</div>
		{:else}
			<div class="grid gap-3">
				{#each submissions as submission}
					<button class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 text-left hover:border-teal-900/25 hover:bg-white" onclick={() => openSubmission(submission.id)} type="button">
						<p class="text-xs uppercase tracking-[0.18em] text-stone-500">{submission.status}</p>
						<h2 class="mt-2 font-serif text-xl text-stone-950">{submission.title}</h2>
						<p class="text-sm text-stone-600">{submission.author}</p>
						<p class="mt-2 text-xs text-stone-500">Requested by {submission.requested_by_username}</p>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<div class="dashboard-card space-y-5">
		{#if !detail}
			<p class="text-sm text-stone-600">Select a submission to inspect its linked requests and event trail.</p>
		{:else}
			<div>
				<p class="eyebrow">Submission detail</p>
				<h2 class="mt-3 font-serif text-3xl text-stone-950">{detail.submission.title}</h2>
				<p class="mt-2 text-sm text-stone-600">{detail.submission.author}</p>
			</div>

			<div class="flex flex-wrap gap-2">
				<span class="status-pill">{detail.submission.status}</span>
				{#if detail.submission.requires_admin_approval}
					<span class="status-pill bg-amber-100 text-amber-900">Needs admin approval</span>
				{/if}
			</div>

			{#if detail.submission.notes}
				<div class="rounded-[1.4rem] bg-stone-50/90 p-4 text-sm text-stone-700">
					<p class="font-medium text-stone-900">Notes</p>
					<p class="mt-2 leading-6">{detail.submission.notes}</p>
				</div>
			{/if}

			{#if detail.submission.linked_requests.length > 0}
				<div class="space-y-3">
					<p class="font-medium text-stone-900">Linked Athena requests</p>
					<div class="grid gap-3">
						{#each detail.submission.linked_requests as request}
							<a class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 hover:border-teal-900/25 hover:bg-white" href={`/requests/${request.id}`}>
								<p class="text-xs uppercase tracking-[0.18em] text-stone-500">{request.state}</p>
								<p class="mt-2 font-semibold text-stone-900">{request.title}</p>
							</a>
						{/each}
					</div>
				</div>
			{/if}

			{#if $authState.user?.role === 'admin'}
				<div class="space-y-4 rounded-[1.5rem] border border-stone-200 bg-stone-50/90 p-5">
					<p class="font-medium text-stone-900">Admin actions</p>
					<div class="flex flex-wrap gap-3">
						<button class="action-button" onclick={approveCurrent} type="button">
							<ShieldCheck class="h-4 w-4" />
							<span>Approve</span>
						</button>
						<button class="ghost-button" onclick={rejectCurrent} type="button">
							<ShieldX class="h-4 w-4" />
							<span>Reject</span>
						</button>
					</div>

					{#if detail.submission.status === 'pending_resolution'}
						<div class="space-y-3">
							<label class="space-y-2 text-sm text-stone-600">
								<span class="font-medium text-stone-800">Resolve against metadata</span>
								<input bind:value={resolveQuery} class="input-shell" placeholder={detail.submission.title} />
							</label>
							<button class="ghost-button" onclick={runResolveSearch} type="button">
								<Search class="h-4 w-4" />
								<span>Search metadata for resolution</span>
							</button>
							<div class="grid gap-3">
								{#each resolveResults as result}
									<button class="rounded-[1.4rem] border border-stone-200 bg-white p-4 text-left hover:border-teal-900/25" disabled={resolving} onclick={() => resolveWith(result.work.external_id)} type="button">
										<p class="font-semibold text-stone-900">{result.work.title}</p>
										<p class="text-sm text-stone-600">{result.work.primary_author}</p>
									</button>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/if}

			<div class="space-y-3">
				<p class="font-medium text-stone-900">Event trail</p>
				<div class="grid gap-3">
					{#each detail.events as event}
						<div class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4">
							<p class="text-xs uppercase tracking-[0.18em] text-stone-500">{event.kind}</p>
							<pre class="mt-2 overflow-x-auto text-xs text-stone-700">{event.payload_json}</pre>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>
