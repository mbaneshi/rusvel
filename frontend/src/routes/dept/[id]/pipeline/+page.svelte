<script lang="ts">
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import { browser } from '$app/environment';
	import {
		getHarvestOpportunities,
		getJobs,
		postHarvestAdvance,
		postHarvestProposal,
		type JobListItem,
		type OpportunityRow
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';

	const STAGES = ['Cold', 'Contacted', 'Qualified', 'ProposalSent', 'Won', 'Lost'] as const;
	const DND_PAYLOAD = 'application/x-rusvel-opp+json';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let rows: OpportunityRow[] = $state([]);
	let loading = $state(true);
	let busy = $state<string | null>(null);
	let busyProposal = $state<string | null>(null);
	/** Freelancer / voice string sent to `POST /api/dept/harvest/proposal` as `profile`. */
	let proposalProfile = $state('default');
	let proposalJobs = $state<JobListItem[]>([]);
	let refreshingJobs = $state(false);
	let dragOverStage = $state<string | null>(null);
	/** Minimum score (0–100) for cards shown in the Kanban; persisted per session in sessionStorage. */
	let minScorePercent = $state(0);

	let deptId = $derived(page.params.id);
	let isHarvest = $derived(deptId === 'harvest');

	function scorePct(row: OpportunityRow): number {
		const s = row.score;
		if (typeof s !== 'number' || Number.isNaN(s)) return 0;
		return s <= 1 ? s * 100 : Math.min(100, s);
	}

	let filteredRows = $derived(
		rows.filter((r) => scorePct(r) >= minScorePercent)
	);

	function normalizeStage(s: string): string {
		const t = s.replace(/"/g, '').trim();
		if (STAGES.includes(t as (typeof STAGES)[number])) return t;
		return 'Cold';
	}

	function stageOf(row: OpportunityRow): string {
		const s = row.stage as unknown;
		if (typeof s === 'string') return normalizeStage(s);
		if (s && typeof s === 'object' && !Array.isArray(s)) {
			const keys = Object.keys(s as object);
			if (keys.length === 1) return normalizeStage(keys[0]);
		}
		return 'Cold';
	}

	function byStage(stage: string): OpportunityRow[] {
		return filteredRows.filter((r) => stageOf(r) === stage);
	}

	function minScoreStorageKey(): string | null {
		if (!sessionId) return null;
		return `rusvel:harvest:${sessionId}:minScorePct`;
	}

	function hydrateMinScoreFromStorage() {
		if (!browser || !isHarvest) return;
		const key = minScoreStorageKey();
		if (!key) return;
		const raw = sessionStorage.getItem(key);
		if (raw === null) {
			minScorePercent = 0;
			return;
		}
		const n = Number.parseInt(raw, 10);
		if (!Number.isNaN(n) && n >= 0 && n <= 100) minScorePercent = n;
	}

	function persistMinScore(v: number) {
		if (!browser) return;
		const key = minScoreStorageKey();
		if (!key) return;
		sessionStorage.setItem(key, String(v));
	}

	function setMinScorePercent(v: number) {
		const clamped = Math.max(0, Math.min(100, Math.round(v)));
		minScorePercent = clamped;
		persistMinScore(clamped);
	}

	async function loadProposalJobs() {
		if (!sessionId || !isHarvest) return;
		try {
			proposalJobs = await getJobs(sessionId, {
				kinds: ['ProposalDraft'],
				statuses: ['Queued', 'Running', 'AwaitingApproval'],
				limit: 24
			});
		} catch {
			proposalJobs = [];
		}
	}

	async function refreshProposalJobs() {
		refreshingJobs = true;
		try {
			await loadProposalJobs();
		} finally {
			refreshingJobs = false;
		}
	}

	async function load() {
		if (!sessionId || !isHarvest) return;
		loading = true;
		try {
			rows = await getHarvestOpportunities(sessionId);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load pipeline');
			rows = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isHarvest) return;
		hydrateMinScoreFromStorage();
		void load();
		void loadProposalJobs();
		const onVis = () => {
			if (!browser || !document.hidden) void loadProposalJobs();
		};
		document.addEventListener('visibilitychange', onVis);
		const t = setInterval(() => {
			if (browser && document.hidden) return;
			void loadProposalJobs();
		}, 12000);
		return () => {
			clearInterval(t);
			document.removeEventListener('visibilitychange', onVis);
		};
	});

	$effect(() => {
		if (!browser) return;
		const clearDrag = () => {
			dragOverStage = null;
		};
		window.addEventListener('dragend', clearDrag);
		return () => window.removeEventListener('dragend', clearDrag);
	});

	function dragStart(e: DragEvent, oppId: string, fromStage: string) {
		if (!e.dataTransfer || busy !== null || busyProposal !== null) {
			e.preventDefault();
			return;
		}
		e.dataTransfer.setData(DND_PAYLOAD, JSON.stringify({ oppId, fromStage }));
		e.dataTransfer.effectAllowed = 'move';
	}

	function dragOverColumn(e: DragEvent, stage: string) {
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'move';
		}
		dragOverStage = stage;
	}

	function dragOverCard(e: DragEvent, stage: string) {
		e.preventDefault();
		e.stopPropagation();
		dragOverColumn(e, stage);
	}

	async function dropOnColumn(e: DragEvent, targetStage: string) {
		e.preventDefault();
		dragOverStage = null;
		const raw = e.dataTransfer?.getData(DND_PAYLOAD);
		if (!raw) return;
		try {
			const { oppId, fromStage } = JSON.parse(raw) as { oppId: string; fromStage: string };
			if (fromStage === targetStage) return;
			await moveTo(oppId, targetStage);
		} catch {
			/* ignore */
		}
	}

	async function dropOnCard(e: DragEvent, targetStage: string) {
		e.preventDefault();
		e.stopPropagation();
		await dropOnColumn(e, targetStage);
	}

	async function moveTo(oppId: string, stage: string) {
		if (!sessionId) return;
		busy = oppId;
		try {
			await postHarvestAdvance(sessionId, oppId, stage);
			toast.success(`Moved to ${stage}`);
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Update failed');
		} finally {
			busy = null;
		}
	}

	async function queueProposal(oppId: string) {
		if (!sessionId) return;
		busyProposal = oppId;
		const profile = proposalProfile.trim() || 'default';
		try {
			const res = await postHarvestProposal(sessionId, oppId, profile);
			if (
				res &&
				typeof res === 'object' &&
				'status' in res &&
				res.status === 'queued' &&
				'job_id' in res &&
				typeof (res as { job_id: unknown }).job_id === 'string'
			) {
				const id = (res as { job_id: string }).job_id;
				toast.success('Proposal queued', {
					description: `Job ${id}. Keep this app running so the job worker can process the queue, then open Approvals.`
				});
			} else {
				toast.success('Proposal request sent.');
			}
			await loadProposalJobs();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Proposal queue failed');
		} finally {
			busyProposal = null;
		}
	}
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
	<div class="border-b border-border px-4 py-3">
		<h1 class="text-lg font-semibold">Opportunity pipeline</h1>
		<p class="text-xs text-muted-foreground">
			Kanban by stage — drag cards between columns or use arrows. Pipeline scan minimum is set in Harvest
			config; use the slider to hide lower-scoring cards here.
		</p>
		{#if isHarvest && sessionId}
			<div class="mt-3 flex flex-col gap-2 sm:max-w-xl">
				<div class="flex flex-wrap items-end gap-3">
					<div class="min-w-[200px] flex-1">
						<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="min-score-range">
							Min. score (show cards at or above)
						</label>
						<div class="flex items-center gap-2">
							<input
								id="min-score-range"
								type="range"
								min="0"
								max="100"
								step="1"
								value={minScorePercent}
								oninput={(e) =>
									setMinScorePercent(Number((e.currentTarget as HTMLInputElement).value))}
								class="h-2 w-full flex-1 cursor-pointer accent-primary"
								aria-valuemin={0}
								aria-valuemax={100}
								aria-valuenow={minScorePercent}
								aria-label="Minimum opportunity score percent"
							/>
							<span class="w-10 shrink-0 text-right font-mono text-xs tabular-nums text-foreground"
								>{minScorePercent}%</span
							>
							<Button
								type="button"
								variant="ghost"
								size="sm"
								class="!h-8 shrink-0 !px-2 !text-[10px]"
								onclick={() => setMinScorePercent(0)}
							>
								Reset
							</Button>
						</div>
					</div>
				</div>
				{#if rows.length > 0}
					<p class="text-[11px] text-muted-foreground">
						Showing <span class="font-medium text-foreground">{filteredRows.length}</span> of
						<span class="tabular-nums">{rows.length}</span> opportunities
						{#if filteredRows.length < rows.length}
							<span class="text-muted-foreground/80">
								(scores below {minScorePercent}% hidden)</span
							>
						{/if}
					</p>
				{/if}
			</div>
			<div class="mt-3 max-w-md">
				<Input
					label="Proposal profile"
					bind:value={proposalProfile}
					size="sm"
					placeholder="default"
					hint="Voice or positioning for generated proposals (queue path)."
				/>
			</div>
			<p class="mt-2 text-[11px] text-muted-foreground">
				Queued proposals run in the background worker (same process as
				<code class="rounded bg-muted px-1 py-0.5 font-mono text-[10px]">cargo run</code>).
				When the job finishes, it appears under
				<a class="text-foreground underline underline-offset-2 hover:text-primary" href="/approvals"
					>Approvals</a
				>.
			</p>
		{/if}
	</div>

	{#if isHarvest && sessionId}
		<div class="border-b border-border bg-muted/30 px-4 py-2">
			<div class="flex items-center justify-between gap-2">
				<p class="text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
					Proposal jobs (queue)
				</p>
				<Button
					variant="ghost"
					size="sm"
					class="!h-7 !px-2 !text-[10px]"
					loading={refreshingJobs}
					disabled={refreshingJobs}
					onclick={() => refreshProposalJobs()}
				>
					Refresh
				</Button>
			</div>
			{#if proposalJobs.length > 0}
				<ul class="mt-1.5 flex flex-wrap gap-2">
					{#each proposalJobs as j}
						<li
							class="rounded border border-border bg-background px-2 py-1 font-mono text-[10px] text-foreground"
						>
							{j.id.slice(0, 8)}… · {j.status}
						</li>
					{/each}
				</ul>
			{:else}
				<p class="mt-1 text-[10px] text-muted-foreground">No proposal jobs in this session.</p>
			{/if}
		</div>
	{/if}

	{#if !isHarvest}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">
				Pipeline is available under <span class="font-mono text-foreground">/dept/harvest/pipeline</span>.
			</p>
		</div>
	{:else if !sessionId}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">Select a session in the top bar.</p>
		</div>
	{:else if loading}
		<div class="p-6 text-sm text-muted-foreground">Loading…</div>
	{:else}
		<div class="min-h-0 flex-1 overflow-x-auto p-3">
			<div class="flex min-w-max gap-3">
				{#each STAGES as stage}
					<div
						role="region"
						aria-label={`${stage} column`}
						class="flex w-56 shrink-0 flex-col rounded-lg border bg-card transition-shadow {dragOverStage ===
						stage
							? 'border-primary ring-2 ring-primary/40'
							: 'border-border'}"
						ondragover={(e) => dragOverColumn(e, stage)}
						ondrop={(e) => dropOnColumn(e, stage)}
					>
						<div
							class="border-b border-border px-2 py-2 text-[11px] font-medium uppercase tracking-wide text-muted-foreground"
						>
							{stage}
							<span class="text-muted-foreground/70">({byStage(stage).length})</span>
						</div>
						<div
							role="list"
							class="flex min-h-24 max-h-[calc(100vh-14rem)] flex-col gap-2 overflow-y-auto p-2"
							ondragover={(e) => dragOverColumn(e, stage)}
							ondrop={(e) => dropOnColumn(e, stage)}
						>
							{#each byStage(stage) as o (o.id)}
								<div
									role="listitem"
									class="rounded-md border border-border bg-secondary/40 p-2 text-xs shadow-sm {busy ===
									o.id
										? 'opacity-60'
										: 'cursor-grab active:cursor-grabbing'}"
									draggable={busy === null && busyProposal === null}
									ondragstart={(e) => dragStart(e, o.id, stage)}
									ondragover={(e) => dragOverCard(e, stage)}
									ondrop={(e) => dropOnCard(e, stage)}
								>
									<p class="font-medium leading-snug text-foreground">{o.title}</p>
									<p class="mt-1 text-[10px] text-muted-foreground">
										<span class="text-foreground/90">Score</span>
										{scorePct(o).toFixed(0)}%
										<span class="mx-1 text-muted-foreground/60">·</span>
										<span class="text-foreground/90">Budget</span>
										{#if o.value_estimate != null && o.value_estimate !== undefined}
											~${o.value_estimate}
										{:else}
											—
										{/if}
									</p>
									<p class="mt-0.5 truncate text-[10px] text-muted-foreground" title={typeof o.source === 'string' ? o.source : JSON.stringify(o.source ?? '')}>
										<span class="text-foreground/80">Source</span>
										{typeof o.source === 'string' ? o.source : JSON.stringify(o.source ?? '')}
									</p>
									<div class="mt-2 flex flex-wrap gap-1">
										<Button
											variant="outline"
											size="sm"
											class="!h-7 !px-1.5 !text-[10px]"
											disabled={busy !== null || busyProposal !== null}
											loading={busyProposal === o.id}
											onclick={() => queueProposal(o.id)}
										>
											Queue proposal
										</Button>
										{#each STAGES as next}
											{#if next !== stage}
												<Button
													variant="secondary"
													size="sm"
													class="!h-7 !px-1.5 !text-[10px]"
													disabled={busy !== null || busyProposal !== null}
													loading={busy === o.id}
													onclick={() => moveTo(o.id, next)}
												>
													→ {next}
												</Button>
											{/if}
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
