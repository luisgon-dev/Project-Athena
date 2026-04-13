<script lang="ts">
	import { onMount } from 'svelte';
	import { ScanLine, RefreshCw, BookOpen, Headphones, CopyX, Clock, CheckCircle2, AlertCircle } from 'lucide-svelte';
	import { triggerLibraryScan, getLibraryScanStatus } from '$lib/api';
	import type { LibraryScanJobRecord } from '$lib/types/LibraryScanJobRecord';

	let job = $state<LibraryScanJobRecord | null>(null);
	let loading = $state(false);
	let error = $state('');

	async function loadStatus() {
		try {
			job = await getLibraryScanStatus();
			error = '';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load scan status';
		}
	}

	async function startScan() {
		loading = true;
		error = '';
		try {
			await triggerLibraryScan();
			// Poll a few times to catch completion
			for (let i = 0; i < 6; i++) {
				await new Promise((r) => setTimeout(r, 800));
				await loadStatus();
				if (job?.completed_at) break;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Scan request failed';
		} finally {
			loading = false;
		}
	}

	function formatStamp(value: string): string {
		return value.replace('T', ' ').replace(/\.\d{3}Z?$/, '');
	}

	onMount(() => {
		void loadStatus();
	});
</script>

<div class="space-y-6">
	<div class="dashboard-card">
		<div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
			<div class="space-y-3">
				<p class="eyebrow">
					<ScanLine class="h-4 w-4" />
					<span>Library scan</span>
				</p>
				<h1 class="font-serif text-3xl font-semibold tracking-tight text-stone-950 sm:text-4xl">
					Catalogue existing books
				</h1>
				<p class="max-w-2xl text-sm leading-6 text-stone-600 sm:text-base">
					Scan the configured ebook and audiobook roots to discover books already on disk.
					New discoveries are added as imported requests and de-duplicated automatically.
				</p>
			</div>
			<div class="flex flex-wrap gap-3">
				<button
					class="action-button bg-teal-900 text-stone-50 hover:bg-teal-800"
					onclick={startScan}
					disabled={loading}
					type="button"
				>
					{#if loading}
						<RefreshCw class="h-4 w-4 animate-spin" />
						<span>Scanning…</span>
					{:else}
						<ScanLine class="h-4 w-4" />
						<span>Start scan</span>
					{/if}
				</button>
				<button
					class="ghost-button border-stone-300 bg-white hover:border-stone-400"
					onclick={loadStatus}
					disabled={loading}
					type="button"
				>
					<RefreshCw class="h-4 w-4" />
					<span>Refresh status</span>
				</button>
			</div>
		</div>
	</div>

	{#if error}
		<div class="dashboard-card border-red-200 bg-red-50 text-red-800">
			<div class="flex items-center gap-3">
				<AlertCircle class="h-5 w-5" />
				<p class="text-sm font-medium">{error}</p>
			</div>
		</div>
	{/if}

	{#if job}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
			<div class="dashboard-card">
				<p class="eyebrow mb-4">
					<BookOpen class="h-4 w-4" />
					<span>Ebooks found</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{Number(job.ebooks_found)}</p>
			</div>
			<div class="dashboard-card">
				<p class="eyebrow mb-4">
					<Headphones class="h-4 w-4" />
					<span>Audiobooks found</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{Number(job.audiobooks_found)}</p>
			</div>
			<div class="dashboard-card">
				<p class="eyebrow mb-4">
					<CopyX class="h-4 w-4" />
					<span>Duplicates skipped</span>
				</p>
				<p class="font-serif text-4xl text-stone-950">{Number(job.duplicates_skipped)}</p>
			</div>
			<div class="dashboard-card">
				<p class="eyebrow mb-4">
					{#if job.completed_at}
						<CheckCircle2 class="h-4 w-4 text-teal-700" />
					{:else}
						<Clock class="h-4 w-4 text-amber-600" />
					{/if}
					<span>Status</span>
				</p>
				<p class="font-serif text-2xl text-stone-950">
					{#if job.completed_at}
						Completed
					{:else}
						In progress
					{/if}
				</p>
				<p class="mt-1 text-xs text-stone-500">
					Started {formatStamp(job.started_at)}
				</p>
			</div>
		</div>

		{#if job.error_message}
			<div class="dashboard-card border-red-200 bg-red-50 text-red-800">
				<div class="flex items-center gap-3">
					<AlertCircle class="h-5 w-5" />
					<div>
						<p class="text-sm font-medium">Last scan failed</p>
						<p class="text-sm opacity-90">{job.error_message}</p>
					</div>
				</div>
			</div>
		{/if}
	{:else}
		<div class="dashboard-card border-dashed border-stone-300 bg-stone-50/80 px-5 py-8 text-sm text-stone-600">
			No scan has been run yet. Start a scan to catalogue books already present in your library.
		</div>
	{/if}
</div>
