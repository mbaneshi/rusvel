<script lang="ts">
	import {
		listArtifacts,
		createArtifact,
		deleteArtifact,
		type ArtifactRecord
	} from '$lib/api';
	import { activeSession } from '$lib/stores';
	import { FileStack } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';

	let items = $state<ArtifactRecord[]>([]);
	let err = $state('');
	let title = $state('');
	let body = $state('');
	let loading = $state(true);

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	async function load() {
		loading = true;
		err = '';
		try {
			items = await listArtifacts();
		} catch (e) {
			err = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function save() {
		if (!title.trim()) {
			toast.error('Title required');
			return;
		}
		try {
			await createArtifact({
				title: title.trim(),
				body,
				kind: 'markdown',
				session_id: sessionId
			});
			title = '';
			body = '';
			toast.success('Artifact saved');
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : String(e));
		}
	}

	async function remove(id: string) {
		try {
			await deleteArtifact(id);
			toast.success('Deleted');
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : String(e));
		}
	}

	load();
</script>

<div class="h-full overflow-auto p-6">
	<div class="mb-6 flex items-center gap-3">
		<FileStack class="h-7 w-7 text-muted-foreground" strokeWidth={1.75} />
		<div>
			<h1 class="text-xl font-semibold text-foreground">Artifacts</h1>
			<p class="text-sm text-muted-foreground">
				Saved outputs from work sessions (Claude-style artifact library).
			</p>
		</div>
	</div>

	<section class="mb-8 max-w-2xl rounded-lg border border-border bg-card p-4">
		<h2 class="mb-3 text-sm font-semibold text-foreground">New artifact</h2>
		<label for="artifact-title" class="mb-2 block text-xs text-muted-foreground">Title</label>
		<input
			id="artifact-title"
			class="mb-3 w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
			bind:value={title}
			placeholder="e.g. Q1 plan draft"
		/>
		<label for="artifact-body" class="mb-2 block text-xs text-muted-foreground">Body (markdown)</label>
		<textarea
			id="artifact-body"
			class="mb-3 min-h-32 w-full rounded-md border border-border bg-background px-3 py-2 font-mono text-xs"
			bind:value={body}
			placeholder="Paste or write content…"
		></textarea>
		<button
			type="button"
			class="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90"
			onclick={() => save()}
		>
			Save artifact
		</button>
	</section>

	{#if loading}
		<p class="text-sm text-muted-foreground">Loading…</p>
	{:else if err}
		<p class="text-sm text-destructive">{err}</p>
	{:else if items.length === 0}
		<p class="text-sm text-muted-foreground">No artifacts yet.</p>
	{:else}
		<ul class="max-w-3xl space-y-4">
			{#each items as a}
				<li class="rounded-lg border border-border bg-card p-4">
					<div class="mb-2 flex flex-wrap items-center justify-between gap-2">
						<h3 class="font-medium text-foreground">{a.title}</h3>
						<button
							type="button"
							class="text-xs text-destructive hover:underline"
							onclick={() => remove(a.id)}
						>
							Delete
						</button>
					</div>
					<p class="mb-2 text-xs text-muted-foreground">
						{a.created_at} · {a.kind}
						{#if a.session_id}
							· session {a.session_id}
						{/if}
					</p>
					<pre class="max-h-48 overflow-auto whitespace-pre-wrap rounded bg-secondary p-3 font-mono text-xs">{a.body}</pre>
				</li>
			{/each}
		</ul>
	{/if}
</div>
