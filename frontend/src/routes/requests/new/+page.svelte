<script lang="ts">
	import { Search, Sparkles, BookOpenText, CheckCircle2, LoaderCircle } from 'lucide-svelte';
	import { createRequests, searchRequests } from '$lib/api';
	import type { CreateRequestSelection } from '$lib/types/CreateRequestSelection';
	import type { MediaType } from '$lib/types/MediaType';
	import type { RequestRecord } from '$lib/types/RequestRecord';
	import type { WorkRecord } from '$lib/types/WorkRecord';

	let title = $state('');
	let author = $state('');
	let selectedWorkId = $state('');
	let preferredLanguage = $state('en');
	let editionTitle = $state('');
	let preferredNarrator = $state('');
	let preferredPublisher = $state('');
	let wantsEbook = $state(true);
	let wantsAudiobook = $state(false);
	let graphicAudio = $state(false);

	let searchResults = $state([] as WorkRecord[]);
	let createdRequests = $state([] as RequestRecord[]);
	let hasSearched = $state(false);
	let searching = $state(false);
	let submitting = $state(false);
	let searchError = $state('');
	let submitError = $state('');

	function coverUrl(coverId: WorkRecord['cover_id']): string {
		return `/api/v1/covers/openlibrary/${String(coverId)}`;
	}

	function buildPayload(): CreateRequestSelection {
		const mediaTypes: MediaType[] = [];
		if (wantsEbook) {
			mediaTypes.push('Ebook');
		}
		if (wantsAudiobook) {
			mediaTypes.push('Audiobook');
		}

		return {
			selected_work_id: selectedWorkId || null,
			media_types: mediaTypes,
			preferred_language: preferredLanguage.trim() || null,
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
		hasSearched = true;
		selectedWorkId = '';
		createdRequests = [];

		try {
			const response = await searchRequests({ title, author });
			searchResults = response.works;
			if (searchResults.length === 1) {
				selectedWorkId = searchResults[0].external_id;
			}
		} catch (error) {
			searchResults = [];
			searchError = error instanceof Error ? error.message : 'Metadata search failed.';
		} finally {
			searching = false;
		}
	}

	async function submitRequest(event: SubmitEvent) {
		event.preventDefault();
		submitting = true;
		submitError = '';

		try {
			createdRequests = await createRequests(buildPayload());
		} catch (error) {
			createdRequests = [];
			submitError = error instanceof Error ? error.message : 'Request creation failed.';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="space-y-6">
	<section class="dashboard-card overflow-hidden bg-[linear-gradient(135deg,rgba(12,74,110,0.96),rgba(21,128,61,0.88))] text-stone-50">
		<div class="grid gap-6 lg:grid-cols-[minmax(0,1.35fr)_minmax(18rem,0.9fr)]">
			<div class="space-y-4">
				<div class="eyebrow border-white/20 bg-white/10 text-stone-50">
					<Sparkles class="h-4 w-4" />
					<span>Metadata-first intake</span>
				</div>
				<div class="space-y-3">
					<h1 class="font-serif text-4xl font-semibold tracking-tight">Metadata Request Wizard</h1>
					<p class="max-w-2xl text-sm leading-6 text-stone-200 sm:text-base">
						Search Open Library first, confirm the canonical work, then commit the final request
						payload to the JSON API with the exact manifestation preferences you care about.
					</p>
				</div>
			</div>
			<div class="rounded-[1.75rem] border border-white/15 bg-white/10 p-5 text-sm text-stone-100 backdrop-blur">
				<p class="font-semibold uppercase tracking-[0.2em] text-stone-200">Flow</p>
				<ol class="mt-4 space-y-3">
					<li>1. Search by title and optional author.</li>
					<li>2. Choose the canonical work record that best matches.</li>
					<li>3. Submit ebook and audiobook preferences in one shot.</li>
				</ol>
			</div>
		</div>
	</section>

	<div class="grid gap-6 xl:grid-cols-[minmax(0,1.25fr)_minmax(20rem,0.95fr)]">
		<form class="dashboard-card space-y-5" onsubmit={runSearch}>
			<div>
				<p class="eyebrow">
					<Search class="h-4 w-4" />
					<span>Step 1</span>
				</p>
				<h2 class="mt-3 font-serif text-2xl text-stone-950">Search the catalog</h2>
			</div>

			<div class="grid gap-4 md:grid-cols-2">
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Title</span>
					<input bind:value={title} class="input-shell" placeholder="The Hobbit" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Author</span>
					<input bind:value={author} class="input-shell" placeholder="Tolkien" />
				</label>
			</div>

			<div class="flex flex-wrap gap-3">
				<button class="action-button" disabled={searching} type="submit">
					{#if searching}
						<LoaderCircle class="h-4 w-4 animate-spin" />
					{:else}
						<Search class="h-4 w-4" />
					{/if}
					<span>{searching ? 'Searching…' : 'Search metadata'}</span>
				</button>
				<p class="self-center text-sm text-stone-500">Leave author blank if the title alone is distinctive.</p>
			</div>

			{#if searchError}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">
					{searchError}
				</div>
			{/if}

			{#if hasSearched && !searching}
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<h3 class="font-serif text-xl text-stone-950">Search results</h3>
						<p class="text-sm text-stone-500">{searchResults.length} candidate works</p>
					</div>

					{#if searchResults.length === 0}
						<div class="rounded-[1.5rem] border border-dashed border-stone-300 bg-stone-50/80 px-5 py-6 text-sm text-stone-600">
							No works matched that search. Adjust the title or author and try again.
						</div>
					{:else}
						<div class="grid gap-4">
							{#each searchResults as work}
								<label class="group block cursor-pointer rounded-[1.65rem] border border-stone-200 bg-stone-50/80 p-4 transition hover:border-teal-800/25 hover:bg-white">
									<div class="grid gap-4 md:grid-cols-[7rem_minmax(0,1fr)]">
										<div class="overflow-hidden rounded-[1.35rem] bg-stone-200/70">
											{#if work.cover_id !== null}
												<img alt={work.title} class="h-full min-h-32 w-full object-cover" src={coverUrl(work.cover_id)} />
											{:else}
												<div class="flex h-full min-h-32 items-center justify-center text-xs uppercase tracking-[0.24em] text-stone-500">
													No cover
												</div>
											{/if}
										</div>
										<div class="space-y-3">
											<div class="flex items-start gap-3">
												<input bind:group={selectedWorkId} class="mt-1 h-4 w-4 accent-teal-900" name="selected-work" type="radio" value={work.external_id} />
												<div class="space-y-2">
													<h4 class="font-serif text-2xl text-stone-950">{work.title}</h4>
													<p class="text-sm text-stone-600">{work.primary_author}</p>
												</div>
											</div>
											<div class="flex flex-wrap gap-2 text-xs uppercase tracking-[0.18em] text-stone-500">
												{#if work.first_publish_year !== null}
													<span class="status-pill bg-white">{work.first_publish_year}</span>
												{/if}
												{#if work.edition_count !== null}
													<span class="status-pill bg-white">{work.edition_count} editions</span>
												{/if}
												<span class="status-pill bg-white">{work.external_id}</span>
											</div>
											{#if work.description}
												<p class="text-sm leading-6 text-stone-600">{work.description}</p>
											{/if}
											{#if work.subjects.length > 0}
												<div class="flex flex-wrap gap-2">
													{#each work.subjects.slice(0, 4) as subject}
														<span class="rounded-full bg-teal-900/8 px-3 py-1 text-xs font-medium text-teal-950">
															{subject}
														</span>
													{/each}
												</div>
											{/if}
										</div>
									</div>
								</label>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</form>

		<form class="dashboard-card space-y-5" onsubmit={submitRequest}>
			<div>
				<p class="eyebrow">
					<BookOpenText class="h-4 w-4" />
					<span>Step 2</span>
				</p>
				<h2 class="mt-3 font-serif text-2xl text-stone-950">Lock in the request payload</h2>
			</div>

			<div class="grid gap-4">
				<div class="grid gap-3 sm:grid-cols-2">
					<label class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 text-sm text-stone-700">
						<div class="flex items-center gap-3">
							<input bind:checked={wantsEbook} class="h-4 w-4 accent-teal-900" type="checkbox" />
							<div>
								<p class="font-semibold text-stone-900">Ebook</p>
								<p class="text-xs text-stone-500">Add the digital text edition to the batch.</p>
							</div>
						</div>
					</label>
					<label class="rounded-[1.4rem] border border-stone-200 bg-stone-50/90 p-4 text-sm text-stone-700">
						<div class="flex items-center gap-3">
							<input bind:checked={wantsAudiobook} class="h-4 w-4 accent-teal-900" type="checkbox" />
							<div>
								<p class="font-semibold text-stone-900">Audiobook</p>
								<p class="text-xs text-stone-500">Request spoken-word import in the same submit.</p>
							</div>
						</div>
					</label>
				</div>

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
					<input bind:value={preferredNarrator} class="input-shell" placeholder="Andy Serkis" />
				</label>
				<label class="space-y-2 text-sm text-stone-600">
					<span class="font-medium text-stone-800">Preferred publisher</span>
					<input bind:value={preferredPublisher} class="input-shell" placeholder="Optional publisher" />
				</label>

				<label class="flex items-center gap-3 rounded-[1.4rem] border border-stone-200 bg-stone-50/90 px-4 py-4 text-sm text-stone-700">
					<input bind:checked={graphicAudio} class="h-4 w-4 accent-teal-900" type="checkbox" />
					<div>
						<p class="font-semibold text-stone-900">Graphic audio preference</p>
						<p class="text-xs text-stone-500">Signal that cinematic audio editions are desirable.</p>
					</div>
				</label>
			</div>

			{#if submitError}
				<div class="rounded-[1.4rem] border border-red-200 bg-red-50 px-4 py-4 text-sm text-red-700">
					{submitError}
				</div>
			{/if}

			<div class="flex flex-wrap gap-3">
				<button class="action-button" disabled={submitting} type="submit">
					{#if submitting}
						<LoaderCircle class="h-4 w-4 animate-spin" />
					{:else}
						<CheckCircle2 class="h-4 w-4" />
					{/if}
					<span>{submitting ? 'Submitting…' : 'Create request batch'}</span>
				</button>
				<p class="self-center text-sm text-stone-500">
					Choose a work result before you submit. The backend will reject empty selections.
				</p>
			</div>

			{#if createdRequests.length > 0}
				<div class="space-y-4 rounded-[1.6rem] border border-teal-900/15 bg-teal-900/6 p-5">
					<div class="flex items-center gap-2 text-teal-950">
						<CheckCircle2 class="h-5 w-5" />
						<h3 class="font-serif text-xl">Created requests</h3>
					</div>
					<div class="grid gap-3">
						{#each createdRequests as request}
							<a class="rounded-[1.4rem] border border-white/70 bg-white/85 px-4 py-4 transition hover:border-teal-900/30 hover:bg-white" href={`/requests/${request.id}`}>
								<div class="flex items-center justify-between gap-4">
									<div>
										<p class="font-semibold text-stone-900">{request.title}</p>
										<p class="text-sm text-stone-600">{request.author}</p>
									</div>
									<span class="status-pill bg-white">{request.media_type}</span>
								</div>
							</a>
						{/each}
					</div>
				</div>
			{/if}
		</form>
	</div>
</div>
