<script lang="ts">
	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import {
		getGtmDeals,
		postGtmDealAdvance,
		type GtmDealRow,
		type GtmDealStage
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import { Receipt, Users } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';

	const STAGES: GtmDealStage[] = [
		'Lead',
		'Qualified',
		'Proposal',
		'Negotiation',
		'Won',
		'Lost'
	];
	const DND_PAYLOAD = 'application/x-rusvel-gtm-deal+json';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let rows: GtmDealRow[] = $state([]);
	let loading = $state(true);
	let busy = $state<string | null>(null);
	let dragOverStage = $state<string | null>(null);

	let deptId = $derived(page.params.id);
	let isGtm = $derived(deptId === 'gtm');

	function stageOf(row: GtmDealRow): GtmDealStage {
		const s = row.stage;
		if (STAGES.includes(s)) return s;
		return 'Lead';
	}

	function byStage(stage: GtmDealStage): GtmDealRow[] {
		return rows.filter((r) => stageOf(r) === stage);
	}

	const money = new Intl.NumberFormat(undefined, {
		style: 'currency',
		currency: 'USD',
		maximumFractionDigits: 0
	});

	function formatWhen(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
		} catch {
			return iso;
		}
	}

	async function load() {
		if (!sessionId || !isGtm) return;
		loading = true;
		try {
			rows = await getGtmDeals(sessionId);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load deals');
			rows = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isGtm) return;
		void load();
	});

	$effect(() => {
		if (!browser) return;
		const clearDrag = () => {
			dragOverStage = null;
		};
		window.addEventListener('dragend', clearDrag);
		return () => window.removeEventListener('dragend', clearDrag);
	});

	function dragStart(e: DragEvent, dealId: string, fromStage: string) {
		if (!e.dataTransfer || busy !== null) {
			e.preventDefault();
			return;
		}
		e.dataTransfer.setData(DND_PAYLOAD, JSON.stringify({ dealId, fromStage }));
		e.dataTransfer.effectAllowed = 'move';
	}

	function dragOverColumn(e: DragEvent, stage: string) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dragOverStage = stage;
	}

	function dragOverCard(e: DragEvent, stage: string) {
		e.preventDefault();
		e.stopPropagation();
		dragOverColumn(e, stage);
	}

	async function dropOnColumn(e: DragEvent, targetStage: GtmDealStage) {
		e.preventDefault();
		dragOverStage = null;
		const raw = e.dataTransfer?.getData(DND_PAYLOAD);
		if (!raw) return;
		try {
			const { dealId, fromStage } = JSON.parse(raw) as {
				dealId: string;
				fromStage: string;
			};
			if (fromStage === targetStage) return;
			await moveTo(dealId, targetStage);
		} catch {
			/* ignore */
		}
	}

	async function dropOnCard(e: DragEvent, targetStage: GtmDealStage) {
		e.preventDefault();
		e.stopPropagation();
		await dropOnColumn(e, targetStage);
	}

	async function moveTo(dealId: string, stage: GtmDealStage) {
		if (!sessionId) return;
		busy = dealId;
		try {
			await postGtmDealAdvance(sessionId, dealId, stage);
			toast.success(`Moved to ${stage}`);
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Update failed');
		} finally {
			busy = null;
		}
	}
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
	<div class="flex flex-wrap items-start justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">Deal pipeline</h1>
			<p class="text-xs text-muted-foreground">
				Kanban by stage — drag cards between columns or use the arrows. Stages match the GTM CRM model
				(Lead → Qualified → Proposal → Negotiation → Won / Lost).
			</p>
		</div>
		{#if isGtm}
			<div class="flex flex-wrap items-center gap-2">
				<a
					href="/dept/gtm/contacts"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					<Users class="h-3.5 w-3.5 opacity-80" strokeWidth={2} />
					Contacts
				</a>
				<a
					href="/dept/gtm/outreach"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Outreach
				</a>
				<a
					href="/dept/gtm/invoices"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					<Receipt class="h-3.5 w-3.5 opacity-80" strokeWidth={2} />
					Invoices
				</a>
			</div>
		{/if}
	</div>

	{#if !isGtm}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">
				Deals are available under <span class="font-mono text-foreground">/dept/gtm/deals</span>.
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
						class="flex w-60 shrink-0 flex-col rounded-lg border bg-card transition-shadow {dragOverStage ===
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
							{#each byStage(stage) as d (d.id)}
								<div
									role="listitem"
									class="rounded-md border border-border bg-secondary/40 p-2 text-xs shadow-sm {busy ===
									d.id
										? 'opacity-60'
										: 'cursor-grab active:cursor-grabbing'}"
									draggable={busy === null}
									ondragstart={(e) => dragStart(e, d.id, stage)}
									ondragover={(e) => dragOverCard(e, stage)}
									ondrop={(e) => dropOnCard(e, stage)}
								>
									<p class="font-medium leading-snug text-foreground">{d.title}</p>
									<p class="mt-1 font-mono text-[11px] text-chart-2 tabular-nums">
										{money.format(d.value)}
									</p>
									<p class="mt-1 text-[10px] text-muted-foreground">
										<span class="text-foreground/85">Contact</span>
										{d.contact_name?.trim() || '—'}
									</p>
									<p class="mt-0.5 text-[10px] text-muted-foreground" title={d.last_activity}>
										<span class="text-foreground/85">Last activity</span>
										{formatWhen(d.last_activity)}
									</p>
									<div class="mt-2 flex flex-wrap gap-1">
										{#each STAGES as next}
											{#if next !== stage}
												<Button
													variant="secondary"
													size="sm"
													class="!h-7 !px-1.5 !text-[10px]"
													disabled={busy !== null}
													loading={busy === d.id}
													onclick={() => moveTo(d.id, next)}
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
