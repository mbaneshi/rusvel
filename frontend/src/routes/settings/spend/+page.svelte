<script lang="ts">
	import { onMount } from 'svelte';
	import { activeSession } from '$lib/stores';
	import { getAnalyticsSpend, type AnalyticsSpendResponse } from '$lib/api';
	import { toast } from 'svelte-sonner';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	let spend: AnalyticsSpendResponse | null = $state(null);
	let loading = $state(true);

	async function load() {
		loading = true;
		try {
			spend = await getAnalyticsSpend(undefined, currentSession?.id ?? null);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load spend');
			spend = null;
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		load();
		return activeSession.subscribe(() => load());
	});
</script>

<div class="p-6">
	<h1 class="mb-2 text-2xl font-bold text-foreground">LLM spend</h1>
	<p class="mb-6 text-sm text-muted-foreground">
		Aggregated from metric store (<code class="rounded bg-muted px-1">llm.cost_usd</code>). Optional
		session scope includes budget hints from session config.
	</p>

	{#if loading}
		<p class="text-muted-foreground">Loading…</p>
	{:else if spend}
		<div class="max-w-2xl space-y-6">
			<div class="rounded-xl border border-border bg-card p-5">
				<h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					Totals
				</h2>
				<p class="text-3xl font-bold text-foreground">${spend.total_usd.toFixed(4)}</p>
				<p class="mt-1 text-xs text-muted-foreground">All departments (or filtered by query)</p>
			</div>

			{#if spend.session_id != null && spend.session_total_usd != null}
				<div
					class="rounded-xl border p-5 {spend.budget_warning
						? 'border-amber-500/50 bg-amber-500/10'
						: 'border-border bg-card'}"
				>
					<h2 class="mb-2 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
						Active session
					</h2>
					<p class="text-lg font-medium text-foreground">
						${spend.session_total_usd.toFixed(4)}
						{#if spend.session_budget_limit_usd != null}
							<span class="text-muted-foreground">
								/ ${spend.session_budget_limit_usd.toFixed(2)} budget</span
							>
						{/if}
					</p>
					{#if spend.budget_warning}
						<p class="mt-2 text-sm text-amber-600 dark:text-amber-400">
							At or above 80% of configured session budget.
						</p>
					{/if}
				</div>
			{/if}

			<div class="rounded-xl border border-border bg-card p-5">
				<h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					By department
				</h2>
				<ul class="space-y-2">
					{#each Object.entries(spend.by_department).sort((a, b) => b[1] - a[1]) as [dept, usd]}
						<li class="flex justify-between rounded-lg bg-secondary/40 px-3 py-2 text-sm">
							<span class="font-mono text-foreground">{dept}</span>
							<span class="text-muted-foreground">${usd.toFixed(4)}</span>
						</li>
					{/each}
				</ul>
			</div>
		</div>
	{:else}
		<p class="text-muted-foreground">No data.</p>
	{/if}
</div>
