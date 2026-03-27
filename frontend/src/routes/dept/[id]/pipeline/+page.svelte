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

	let deptId = $derived(page.params.id);
	let isHarvest = $derived(deptId === 'harvest');

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
		return rows.filter((r) => stageOf(r) === stage);
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
			Kanban by stage. Minimum score filter: use Harvest config; re-score in Engine tab.
		</p>
		{#if isHarvest && sessionId}
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
					<div class="flex w-56 shrink-0 flex-col rounded-lg border border-border bg-card">
						<div
							class="border-b border-border px-2 py-2 text-[11px] font-medium uppercase tracking-wide text-muted-foreground"
						>
							{stage}
							<span class="text-muted-foreground/70">({byStage(stage).length})</span>
						</div>
						<div class="flex max-h-[calc(100vh-14rem)] flex-col gap-2 overflow-y-auto p-2">
							{#each byStage(stage) as o (o.id)}
								<div class="rounded-md border border-border bg-secondary/40 p-2 text-xs shadow-sm">
									<p class="font-medium leading-snug text-foreground">{o.title}</p>
									<p class="mt-1 text-[10px] text-muted-foreground">
										score {(o.score * 100).toFixed(0)}%
										{#if o.value_estimate != null && o.value_estimate !== undefined}
											· ~${o.value_estimate}
										{/if}
									</p>
									<p class="mt-0.5 truncate text-[10px] text-muted-foreground">
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
