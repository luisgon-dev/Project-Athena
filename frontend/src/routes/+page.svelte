<script lang="ts">
	import { onMount } from 'svelte';
	import { ClipboardList, LoaderCircle, ShieldAlert, Sparkles } from 'lucide-svelte';
	import { listRequests, listSubmissions } from '$lib/api';
	import { authState } from '$lib/auth';
	import type { RequestListRecord } from '$lib/types/RequestListRecord';
	import type { RequestSubmissionRecord } from '$lib/types/RequestSubmissionRecord';

	let requests = $state([] as RequestListRecord[]);
	let submissions = $state([] as RequestSubmissionRecord[]);
	let loading = $state(true);
	let error = $state('');

	const pendingSubmissions = $derived(
		submissions.filter(
			(submission) =>
				submission.status === 'pending_resolution' ||
				(submission.requires_admin_approval && submission.status !== 'approved')
		)
	);

	async function loadAdminDashboard() {
		if ($authState.user?.role !== 'admin') {
			loading = false;
			return;
		}

		loading = true;
		error = '';
		try {
			[submissions, requests] = await Promise.all([listSubmissions(true), listRequests()]);
		} catch (loadError) {
			error =
				loadError instanceof Error ? loadError.message : 'The admin queue could not be loaded.';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadAdminDashboard();
	});
</script>

{#if $authState.user?.role !== 'admin'}
	<div class="dashboard-card">
		<p class="eyebrow">Requester portal</p>
		<h1 class="mt-3 font-serif text-3xl text-stone-950">Admin queue is restricted</h1>
		<p class="mt-3 text-sm leading-6 text-stone-600">
			Use the request wizard to submit books and the My Requests page to track your submissions.
		</p>
	</div>
{:else}
	<div class="space-y-6">
		<section class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(15,23,42,0.98),rgba(19,78,74,0.92))] text-stone-50">
			<div class="space-y-4">
				<div class="eyebrow border-white/20 bg-white/10 text-stone-50">
					<Sparkles class="h-4 w-4" />
					<span>Comparable v1 admin queue</span>
				</div>
				<h1 class="font-serif text-4xl tracking-tight">Submission and fulfillment queue</h1>
				<p class="max-w-3xl text-sm leading-6 text-stone-200">
					Review requester intake, resolve manual submissions, and drill into canonical Athena
					requests when fulfillment starts.
				</p>
			</div>
		</section>

		{#if loading}
			<div class="dashboard-card flex items-center gap-3 text-sm text-stone-600">
				<LoaderCircle class="h-5 w-5 animate-spin text-teal-900" />
				<span>Loading the admin queue…</span>
			</div>
		{:else if error}
			<div class="dashboard-card border border-red-200 bg-red-50 text-sm text-red-700">{error}</div>
		{:else}
			<div class="grid gap-4 md:grid-cols-3">
				<div class="dashboard-card">
					<p class="eyebrow mb-4">
						<ShieldAlert class="h-4 w-4" />
						<span>Needs action</span>
					</p>
					<p class="font-serif text-4xl text-stone-950">{pendingSubmissions.length}</p>
					<p class="mt-2 text-sm text-stone-600">Manual resolution or approval work waiting on admins.</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4">
						<ClipboardList class="h-4 w-4" />
						<span>Submissions</span>
					</p>
					<p class="font-serif text-4xl text-stone-950">{submissions.length}</p>
					<p class="mt-2 text-sm text-stone-600">Total requester submissions across the portal.</p>
				</div>
				<div class="dashboard-card">
					<p class="eyebrow mb-4">
						<ClipboardList class="h-4 w-4" />
						<span>Requests</span>
					</p>
					<p class="font-serif text-4xl text-stone-950">{requests.length}</p>
					<p class="mt-2 text-sm text-stone-600">Canonical fulfillment records active in Athena.</p>
				</div>
			</div>

			<div class="grid gap-6 xl:grid-cols-2">
				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">Pending intake</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Submissions awaiting action</h2>
					</div>
					{#if pendingSubmissions.length === 0}
						<p class="text-sm text-stone-600">No submissions currently need admin work.</p>
					{:else}
						<div class="grid gap-3">
							{#each pendingSubmissions as submission}
								<a class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 hover:border-teal-900/25 hover:bg-white" href={`/my-requests?submission=${submission.id}`}>
									<p class="text-xs uppercase tracking-[0.18em] text-stone-500">{submission.status}</p>
									<h3 class="mt-2 font-serif text-xl text-stone-950">{submission.title}</h3>
									<p class="text-sm text-stone-600">{submission.author}</p>
									<p class="mt-2 text-sm text-stone-500">
										Requested by {submission.requested_by_username}
									</p>
								</a>
							{/each}
						</div>
					{/if}
				</div>

				<div class="dashboard-card space-y-4">
					<div>
						<p class="eyebrow">Fulfillment records</p>
						<h2 class="mt-3 font-serif text-2xl text-stone-950">Recent Athena requests</h2>
					</div>
					<div class="grid gap-3">
						{#each requests.slice(0, 8) as request}
							<a class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 hover:border-teal-900/25 hover:bg-white" href={`/requests/${request.id}`}>
								<p class="text-xs uppercase tracking-[0.18em] text-stone-500">{request.state}</p>
								<h3 class="mt-2 font-serif text-xl text-stone-950">{request.title}</h3>
								<p class="text-sm text-stone-600">{request.author}</p>
							</a>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>
{/if}
