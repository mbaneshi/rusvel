<script lang="ts">
	import { onMount } from 'svelte';
	import {
		ingestKnowledge,
		getKnowledge,
		deleteKnowledge,
		searchKnowledge,
		getKnowledgeStats
	} from '$lib/api';
	import type { KnowledgeEntry, KnowledgeSearchResult, KnowledgeStats } from '$lib/api';
	import { toast } from 'svelte-sonner';

	// Ingest panel state
	let ingestContent = $state('');
	let ingestSource = $state('');
	let ingesting = $state(false);

	// Browse panel state
	let entries: KnowledgeEntry[] = $state([]);
	let loadingEntries = $state(false);

	// Search panel state
	let searchQuery = $state('');
	let searchResults: KnowledgeSearchResult[] = $state([]);
	let searching = $state(false);

	// Stats
	let stats: KnowledgeStats | null = $state(null);
	let statsError = $state('');

	onMount(async () => {
		await Promise.all([loadEntries(), loadStats()]);
	});

	async function loadEntries() {
		loadingEntries = true;
		try {
			entries = await getKnowledge();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load knowledge entries');
		} finally {
			loadingEntries = false;
		}
	}

	async function loadStats() {
		statsError = '';
		try {
			stats = await getKnowledgeStats();
		} catch (e) {
			statsError = e instanceof Error ? e.message : 'Stats unavailable';
		}
	}

	async function handleIngest() {
		if (!ingestContent.trim()) {
			toast.error('Content is required');
			return;
		}
		ingesting = true;
		try {
			const result = await ingestKnowledge(ingestContent, ingestSource || 'manual');
			toast.success(`Ingested ${result.chunks_stored} chunk(s)`);
			ingestContent = '';
			ingestSource = '';
			await Promise.all([loadEntries(), loadStats()]);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Ingest failed');
		} finally {
			ingesting = false;
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteKnowledge(id);
			entries = entries.filter((e) => e.id !== id);
			toast.success('Entry deleted');
			await loadStats();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Delete failed');
		}
	}

	async function handleSearch() {
		if (!searchQuery.trim()) return;
		searching = true;
		try {
			searchResults = await searchKnowledge(searchQuery, 10);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Search failed');
		} finally {
			searching = false;
		}
	}

	function formatDate(iso: string): string {
		try {
			return new Date(iso).toLocaleString([], {
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return iso;
		}
	}

	function truncate(text: string, maxLen: number): string {
		return text.length > maxLen ? text.slice(0, maxLen) + '...' : text;
	}
</script>

<div class="h-full overflow-y-auto p-6">
	<h1 class="mb-6 text-2xl font-bold text-foreground">Knowledge Base</h1>

	<!-- Stats Bar -->
	{#if stats}
		<div class="mb-6 grid grid-cols-3 gap-3">
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium text-muted-foreground">Total Entries</p>
				<p class="mt-1 text-2xl font-bold text-chart-1">{stats.total_entries}</p>
			</div>
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium text-muted-foreground">Embedding Model</p>
				<p class="mt-1 text-sm font-semibold text-foreground">{stats.model_name}</p>
			</div>
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium text-muted-foreground">Dimensions</p>
				<p class="mt-1 text-2xl font-bold text-chart-2">{stats.dimensions}</p>
			</div>
		</div>
	{:else if statsError}
		<div
			class="mb-6 rounded-xl border border-yellow-500/30 bg-yellow-500/10 p-4 text-sm text-yellow-600"
		>
			{statsError}
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<!-- Ingest Panel -->
		<div class="rounded-xl border border-border bg-card p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
				Ingest Knowledge
			</h3>
			<div class="space-y-3">
				<textarea
					bind:value={ingestContent}
					placeholder="Paste text, documentation, notes, or any knowledge to ingest..."
					rows="6"
					class="w-full rounded-lg border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
				></textarea>
				<input
					bind:value={ingestSource}
					placeholder="Source (e.g. docs, notes, article)"
					class="w-full rounded-lg border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
				/>
				<button
					onclick={handleIngest}
					disabled={ingesting || !ingestContent.trim()}
					class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
				>
					{#if ingesting}
						<span class="flex items-center gap-2">
							<span
								class="h-4 w-4 animate-spin rounded-full border-2 border-primary-foreground/30 border-t-primary-foreground"
							></span>
							Ingesting...
						</span>
					{:else}
						Ingest
					{/if}
				</button>
			</div>
		</div>

		<!-- Search Panel -->
		<div class="rounded-xl border border-border bg-card p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
				Semantic Search
			</h3>
			<div class="mb-3 flex gap-2">
				<input
					bind:value={searchQuery}
					placeholder="Search your knowledge base..."
					onkeydown={(e) => {
						if (e.key === 'Enter') handleSearch();
					}}
					class="flex-1 rounded-lg border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
				/>
				<button
					onclick={handleSearch}
					disabled={searching || !searchQuery.trim()}
					class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
				>
					{searching ? 'Searching...' : 'Search'}
				</button>
			</div>
			{#if searchResults.length > 0}
				<div class="space-y-2 max-h-80 overflow-y-auto">
					{#each searchResults as result}
						<div class="rounded-lg bg-secondary/50 p-3">
							<div class="mb-1 flex items-center justify-between">
								<span
									class="rounded-full bg-chart-1/15 px-2 py-0.5 text-xs font-medium text-chart-1"
								>
									score: {result.score.toFixed(3)}
								</span>
								<span class="text-xs text-muted-foreground">{result.entry.source}</span>
							</div>
							<p class="text-sm text-foreground/80">{truncate(result.entry.content, 200)}</p>
						</div>
					{/each}
				</div>
			{:else if searchQuery && !searching}
				<p class="text-sm text-muted-foreground">No results. Try a different query.</p>
			{/if}
		</div>

		<!-- Browse Panel (full width) -->
		<div class="rounded-xl border border-border bg-card p-5 lg:col-span-2">
			<div class="mb-4 flex items-center justify-between">
				<h3 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					All Entries ({entries.length})
				</h3>
				<button
					onclick={loadEntries}
					disabled={loadingEntries}
					class="text-xs text-primary hover:text-primary/80"
				>
					{loadingEntries ? 'Loading...' : 'Refresh'}
				</button>
			</div>
			{#if loadingEntries}
				<div class="flex items-center gap-3 text-muted-foreground">
					<div
						class="h-5 w-5 animate-spin rounded-full border-2 border-border border-t-primary"
					></div>
					Loading...
				</div>
			{:else if entries.length === 0}
				<div class="flex flex-col items-center py-8 text-center">
					<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-primary/15">
						<svg
							class="h-5 w-5 text-primary"
							viewBox="0 0 16 16"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
						>
							<path d="M8 3v10M3 8h10" stroke-linecap="round" />
						</svg>
					</div>
					<p class="text-sm text-muted-foreground">No knowledge entries yet</p>
					<p class="mt-1 text-xs text-muted-foreground/60">
						Use the ingest panel to add text, docs, or notes
					</p>
				</div>
			{:else}
				<div class="space-y-1 max-h-96 overflow-y-auto">
					{#each entries as entry}
						<div class="group flex items-start gap-3 rounded-md px-2 py-2 hover:bg-secondary/50">
							<div class="flex-1 min-w-0">
								<div class="mb-0.5 flex items-center gap-2">
									<span
										class="rounded bg-secondary px-1.5 py-0.5 text-xs font-mono text-muted-foreground"
									>
										{entry.source}
									</span>
									<span class="text-xs text-muted-foreground">{formatDate(entry.created_at)}</span>
								</div>
								<p class="text-sm text-foreground/80 truncate">{truncate(entry.content, 150)}</p>
							</div>
							<button
								onclick={() => handleDelete(entry.id)}
								class="invisible rounded p-1 text-muted-foreground hover:bg-destructive/15 hover:text-destructive group-hover:visible"
								title="Delete entry"
							>
								<svg
									class="h-4 w-4"
									viewBox="0 0 16 16"
									fill="none"
									stroke="currentColor"
									stroke-width="1.5"
								>
									<path d="M4 4l8 8M12 4l-8 8" stroke-linecap="round" />
								</svg>
							</button>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
