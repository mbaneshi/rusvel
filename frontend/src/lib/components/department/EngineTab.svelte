<script lang="ts">
	import { activeSession } from '$lib/stores';
	import {
		getCodeSearch,
		getContentList,
		getHarvestPipeline,
		postCodeAnalyze,
		postContentDraft,
		postForgePipeline
	} from '$lib/api';
	import { toast } from 'svelte-sonner';

	let { dept, deptHsl }: { dept: string; deptHsl: string } = $props();

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	let busy = $state(false);
	let output = $state('');
	let codePath = $state('.');
	let codeQuery = $state('');
	let codeLimit = $state(20);
	let contentTopic = $state('');
	let contentKind = $state('Blog');
	let forgeDraftTopic = $state('');

	function showJson(data: unknown) {
		output = JSON.stringify(data, null, 2);
	}

	async function runCodeAnalyze() {
		busy = true;
		output = '';
		try {
			const r = await postCodeAnalyze(codePath.trim() || '.');
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}

	async function runCodeSearch() {
		if (!codeQuery.trim()) return;
		busy = true;
		output = '';
		try {
			const r = await getCodeSearch(codeQuery.trim(), codeLimit);
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}

	async function runContentDraft() {
		if (!currentSession) {
			toast.error('Select a session first');
			return;
		}
		if (!contentTopic.trim()) return;
		busy = true;
		output = '';
		try {
			const r = await postContentDraft(
				currentSession.id,
				contentTopic.trim(),
				contentKind || undefined
			);
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}

	async function runContentList() {
		if (!currentSession) {
			toast.error('Select a session first');
			return;
		}
		busy = true;
		output = '';
		try {
			const r = await getContentList(currentSession.id);
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}

	async function runHarvestPipeline() {
		if (!currentSession) {
			toast.error('Select a session first');
			return;
		}
		busy = true;
		output = '';
		try {
			const r = await getHarvestPipeline(currentSession.id);
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}

	async function runForgePipeline() {
		if (!currentSession) {
			toast.error('Select a session first');
			return;
		}
		busy = true;
		output = '';
		try {
			const topic = forgeDraftTopic.trim();
			const r = await postForgePipeline(
				currentSession.id,
				topic ? { draft_topic: topic } : undefined
			);
			showJson(r);
		} catch (e) {
			output = e instanceof Error ? e.message : String(e);
			toast.error(output);
		} finally {
			busy = false;
		}
	}
</script>

<div class="p-3 space-y-3 text-[10px]">
	{#if dept === 'code'}
		<div class="space-y-2">
			<p class="font-medium text-foreground" style="color: hsl({deptHsl})">Code engine</p>
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Path</span>
				<input
					bind:value={codePath}
					class="w-full rounded border border-border bg-background px-2 py-1 font-mono text-[10px]"
				/>
			</label>
			<button
				type="button"
				disabled={busy}
				onclick={runCodeAnalyze}
				class="w-full rounded-md py-1.5 text-xs font-medium text-white disabled:opacity-50"
				style="background: hsl({deptHsl})"
			>
				Analyze
			</button>
		</div>
		<div class="space-y-2 border-t border-border pt-2">
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Search query</span>
				<input
					bind:value={codeQuery}
					class="w-full rounded border border-border bg-background px-2 py-1 font-mono text-[10px]"
				/>
			</label>
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Limit</span>
				<input
					type="number"
					bind:value={codeLimit}
					min="1"
					max="100"
					class="w-full rounded border border-border bg-background px-2 py-1"
				/>
			</label>
			<button
				type="button"
				disabled={busy}
				onclick={runCodeSearch}
				class="w-full rounded-md bg-secondary py-1.5 text-xs font-medium disabled:opacity-50"
			>
				Search
			</button>
		</div>
	{:else if dept === 'content'}
		<div class="space-y-2">
			<p class="font-medium text-foreground" style="color: hsl({deptHsl})">Content engine</p>
			{#if !currentSession}
				<p class="text-muted-foreground">Select a session in the header to draft or list.</p>
			{:else}
				<p class="text-muted-foreground">Session: {currentSession.name}</p>
			{/if}
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Topic</span>
				<input
					bind:value={contentTopic}
					class="w-full rounded border border-border bg-background px-2 py-1"
				/>
			</label>
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Kind</span>
				<select
					bind:value={contentKind}
					class="w-full rounded border border-border bg-background px-2 py-1"
				>
					<option value="Blog">Blog</option>
					<option value="LinkedInPost">LinkedInPost</option>
					<option value="Thread">Thread</option>
					<option value="Tweet">Tweet</option>
					<option value="LongForm">LongForm</option>
				</select>
			</label>
			<button
				type="button"
				disabled={busy || !currentSession}
				onclick={runContentDraft}
				class="w-full rounded-md py-1.5 text-xs font-medium text-white disabled:opacity-50"
				style="background: hsl({deptHsl})"
			>
				Draft
			</button>
			<button
				type="button"
				disabled={busy || !currentSession}
				onclick={runContentList}
				class="w-full rounded-md bg-secondary py-1.5 text-xs font-medium disabled:opacity-50"
			>
				List content
			</button>
		</div>
	{:else if dept === 'forge'}
		<div class="space-y-2">
			<p class="font-medium text-foreground" style="color: hsl({deptHsl})">Forge — opportunity pipeline</p>
			{#if !currentSession}
				<p class="text-muted-foreground">Select a session in the header.</p>
			{:else}
				<p class="text-muted-foreground">Session: {currentSession.name}</p>
			{/if}
			<label class="block space-y-0.5">
				<span class="text-muted-foreground">Optional draft topic (overrides harvest title)</span>
				<input
					bind:value={forgeDraftTopic}
					placeholder="Leave empty to use first scanned opportunity title"
					class="w-full rounded border border-border bg-background px-2 py-1"
				/>
			</label>
			<button
				type="button"
				disabled={busy || !currentSession}
				onclick={runForgePipeline}
				class="w-full rounded-md py-1.5 text-xs font-medium text-white disabled:opacity-50"
				style="background: hsl({deptHsl})"
			>
				Run opportunity pipeline
			</button>
			<p class="text-muted-foreground leading-relaxed">
				<code class="rounded bg-muted px-0.5">POST /api/forge/pipeline</code> — Harvest scan → score → propose →
				content draft.
			</p>
		</div>
	{:else if dept === 'harvest'}
		<div class="space-y-2">
			<p class="font-medium text-foreground" style="color: hsl({deptHsl})">Harvest engine</p>
			{#if !currentSession}
				<p class="text-muted-foreground">Select a session in the header.</p>
			{/if}
			<button
				type="button"
				disabled={busy || !currentSession}
				onclick={runHarvestPipeline}
				class="w-full rounded-md py-1.5 text-xs font-medium text-white disabled:opacity-50"
				style="background: hsl({deptHsl})"
			>
				Refresh pipeline
			</button>
		</div>
	{:else}
		<p class="text-muted-foreground">No engine tools for this department.</p>
	{/if}

	{#if output}
		<pre
			class="max-h-64 overflow-auto rounded-md border border-border bg-muted/30 p-2 font-mono text-[9px] leading-relaxed whitespace-pre-wrap break-all"
		>{output}</pre>
	{/if}
</div>
