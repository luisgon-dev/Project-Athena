<script lang="ts">
	import { Search, Sparkles, LoaderCircle, AlertTriangle } from 'lucide-svelte';
	import { authState } from '$lib/auth';
	import { createSubmission, searchSubmissionCandidates } from '$lib/api';
	import type { CreateSubmissionRequest } from '$lib/types/CreateSubmissionRequest';
	import type { MediaType } from '$lib/types/MediaType';
	import type { RequestSubmissionDetailRecord } from '$lib/types/RequestSubmissionDetailRecord';
	import type { SubmissionSearchCandidate } from '$lib/types/SubmissionSearchCandidate';

	let title = $state('');
	let author = $state('');
	let selectedWorkId = $state('');
	let preferredLanguage = $state('en');
	let editionTitle = $state('');
	let preferredNarrator = $state('');
	let preferredPublisher = $state('');
	let notes = $state('');
	let wantsEbook = $state(true);
	let wantsAudiobook = $state(false);
	let graphicAudio = $state(false);
	let allowDuplicate = $state(false);

	let searchResults = $state([] as SubmissionSearchCandidate[]);
	let createdSubmission = $state(null as RequestSubmissionDetailRecord | null);
	let searching = $state(false);
	let submitting = $state(false);
	let searchError = $state('');
	let submitError = $state('');

	function mediaTypes(): MediaType[] {
		const selected: MediaType[] = [];
		if (wantsEbook) selected.push('Ebook');
		if (wantsAudiobook) selected.push('Audiobook');
		return selected;
	}

	function buildMetadataPayload(): CreateSubmissionRequest {
		return {
			intake_mode: 'metadata',
			selected_work_id: selectedWorkId || null,
			title: null,
			author: null,
			media_types: mediaTypes(),
			preferred_language: preferredLanguage.trim() || null,
			notes: notes.trim() || null,
			allow_duplicate: allowDuplicate,
			manifestation: {
				edition_title: editionTitle.trim() || null,
				preferred_narrator: preferredNarrator.trim() || null,
				preferred_publisher: preferredPublisher.trim() || null,
				graphic_audio: graphicAudio
			}
		};
	}

	function buildManualPayload(): CreateSubmissionRequest {
		return {
			intake_mode: 'manual',
			selected_work_id: null,
			title: title.trim() || null,
			author: author.trim() || null,
			media_types: mediaTypes(),
			preferred_language: preferredLanguage.trim() || null,
			notes: notes.trim() || null,
			allow_duplicate: false,
			manifestation: {
				edition_title: editionTitle.trim() || null,
				preferred_narrator: preferredNarrator.trim() || null,
				preferred_publisher: preferredPublisher.trim() || null,
				graphic_audio: graphicAudio
			}
		};
	}

	async function runSearch(event: SubmitEvent) {
		event.preventDefault();
		searching = true;
		searchError = '';
		createdSubmission = null;
		selectedWorkId = '';

		try {
			const response = await searchSubmissionCandidates({ title, author });
			searchResults = response.works;
			if (searchResults.length === 1) {
				selectedWorkId = searchResults[0].work.external_id;
			}
		} catch (error) {
			searchResults = [];
			searchError = error instanceof Error ? error.message : 'Search failed.';
		} finally {
			searching = false;
		}
	}

	async function submitMetadata(event: SubmitEvent) {
		event.preventDefault();
		submitting = true;
		submitError = '';
		try {
			createdSubmission = await createSubmission(buildMetadataPayload());
		} catch (error) {
			submitError = error instanceof Error ? error.message : 'Submission failed.';
		} finally {
			submitting = false;
		}
	}

	async function submitManual() {
		submitting = true;
		submitError = '';
		try {
			createdSubmission = await createSubmission(buildManualPayload());
		} catch (error) {
			submitError = error instanceof Error ? error.message : 'Manual submission failed.';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="space-y-6">
	<section class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(12,74,110,0.96),rgba(21,128,61,0.88))] text-stone-50">
		<div class="space-y-4">
			<div class="eyebrow border-white/20 bg-white/10 text-stone-50">
				<Sparkles class="h-4 w-4" />
				<span>Requester portal</span>
			</div>
			<h1 class="font-serif text-4xl tracking-tight">Search, request, and fall back to manual intake</h1>
			<p class="max-w-3xl text-sm leading-6 text-stone-200">
				Athena now starts with Open Library metadata, shows duplicate hints from Athena and
				Audiobookshelf, and lets admins resolve manual requests when the metadata search misses.
			</p>
		</div>
	</section>

	<div class="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.95fr)]">
		<form class="dashboard-card space-y-5" onsubmit={runSearch}>
			<div>
				<p class="eyebrow">
					<Search class="h-4 w-4" />
					<span>Search</span>
				</p>
				<h2 class="mt-3 font-serif text-2xl text-stone-950">Find the canonical work</h2>
			</div>

			<div class="grid gap-4 md:grid-cols-2">
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Title</span>
					<input bind:value={title} class="input-shell" placeholder="The Hobbit" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Author</span>
					<input bind:value={author} class="input-shell" placeholder="J.R.R. Tolkien" />
				</label>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<label class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 text-sm text-stone-700">
					<div class="flex items-center gap-3">
						<input bind:checked={wantsEbook} class="h-4 w-4 accent-teal-900" type="checkbox" />
						<div>
							<p class="font-semibold text-stone-900">Ebook</p>
							<p class="text-xs text-stone-500">Request text editions.</p>
						</div>
					</div>
				</label>
				<label class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 text-sm text-stone-700">
					<div class="flex items-center gap-3">
						<input bind:checked={wantsAudiobook} class="h-4 w-4 accent-teal-900" type="checkbox" />
						<div>
							<p class="font-semibold text-stone-900">Audiobook</p>
							<p class="text-xs text-stone-500">Request spoken-word editions.</p>
						</div>
					</div>
				</label>
			</div>

			<div class="grid gap-4 md:grid-cols-2">
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Preferred language</span>
					<input bind:value={preferredLanguage} class="input-shell" placeholder="en" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Edition title</span>
					<input bind:value={editionTitle} class="input-shell" placeholder="Optional edition title" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Preferred narrator</span>
					<input bind:value={preferredNarrator} class="input-shell" placeholder="Optional narrator" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Preferred publisher</span>
					<input bind:value={preferredPublisher} class="input-shell" placeholder="Optional publisher" />
				</label>
			</div>

			<label class="space-y-2 text-sm text-stone-600">
				<span class="font-medium text-stone-800">Notes</span>
				<textarea bind:value={notes} class="input-shell min-h-28" placeholder="Anything the admin should know?"></textarea>
			</label>

			<label class="flex items-center gap-3 text-sm text-stone-700">
				<input bind:checked={graphicAudio} class="h-4 w-4 accent-teal-900" type="checkbox" />
				<span>Prefer GraphicAudio / dramatized releases when possible</span>
			</label>

			{#if $authState.user?.role === 'admin'}
				<label class="flex items-center gap-3 text-sm text-stone-700">
					<input bind:checked={allowDuplicate} class="h-4 w-4 accent-teal-900" type="checkbox" />
					<span>Allow duplicate audiobook requests</span>
				</label>
			{/if}

			<div class="flex flex-wrap gap-3">
				<button class="action-button" disabled={searching} type="submit">
					{#if searching}
						<LoaderCircle class="h-4 w-4 animate-spin" />
					{:else}
						<Search class="h-4 w-4" />
					{/if}
					<span>{searching ? 'Searching…' : 'Search metadata'}</span>
				</button>
				<button class="ghost-button" disabled={submitting} onclick={submitManual} type="button">
					<AlertTriangle class="h-4 w-4" />
					<span>Submit manual fallback</span>
				</button>
			</div>

			{#if searchError}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">
					{searchError}
				</div>
			{/if}

			{#if searchResults.length > 0}
				<div class="space-y-4">
					<h3 class="font-serif text-xl text-stone-950">Search results</h3>
					<div class="grid gap-4">
						{#each searchResults as result}
							<label class="rounded-[1.5rem] border border-stone-200 bg-stone-50/90 p-4 hover:border-teal-900/25 hover:bg-white">
								<div class="flex items-start gap-3">
									<input bind:group={selectedWorkId} class="mt-1 h-4 w-4 accent-teal-900" type="radio" value={result.work.external_id} />
									<div class="space-y-2">
										<h4 class="font-serif text-2xl text-stone-950">{result.work.title}</h4>
										<p class="text-sm text-stone-600">{result.work.primary_author}</p>
										{#if result.duplicate_hints.length > 0}
											<div class="flex flex-wrap gap-2">
												{#each result.duplicate_hints as hint}
													<span class="rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-900">
														{hint.label}
													</span>
												{/each}
											</div>
										{/if}
									</div>
								</div>
							</label>
						{/each}
					</div>
				</div>
			{/if}
		</form>

		<form class="dashboard-card space-y-5" onsubmit={submitMetadata}>
			<div>
				<p class="eyebrow">
					<Sparkles class="h-4 w-4" />
					<span>Submit</span>
				</p>
				<h2 class="mt-3 font-serif text-2xl text-stone-950">Create the submission</h2>
			</div>

			<p class="text-sm leading-6 text-stone-600">
				Requesters create submissions immediately. If you selected manual fallback, Athena will hold
				it for admin resolution before it creates canonical fulfillment records.
			</p>

			<button class="action-button w-full justify-center" disabled={submitting || !selectedWorkId} type="submit">
				{#if submitting}
					<LoaderCircle class="h-4 w-4 animate-spin" />
				{:else}
					<Sparkles class="h-4 w-4" />
				{/if}
				<span>{submitting ? 'Submitting…' : 'Submit metadata-backed request'}</span>
			</button>

			{#if submitError}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">
					{submitError}
				</div>
			{/if}

			{#if createdSubmission}
				<div class="rounded-[1.5rem] border border-teal-200 bg-teal-50 px-5 py-5 text-sm text-teal-950">
					<p class="font-semibold">Submission created</p>
					<p class="mt-2">
						{createdSubmission.submission.title} is now tracked as
						<strong>{createdSubmission.submission.status}</strong>.
					</p>
					<a class="mt-3 inline-flex text-sm font-medium text-teal-900 underline" href="/my-requests">
						Open My Requests
					</a>
				</div>
			{/if}
		</form>
	</div>
</div>
