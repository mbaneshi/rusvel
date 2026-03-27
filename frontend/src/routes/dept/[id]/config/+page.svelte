<script lang="ts">
	import { page } from '$app/state';
	import { getAnalyticsSpend, getDeptConfig } from '$lib/api';
	import type { AnalyticsSpendResponse, DepartmentConfig } from '$lib/api';
	import { activeSession } from '$lib/stores';

	let config: DepartmentConfig | null = $state(null);
	let spend: AnalyticsSpendResponse | null = $state(null);
	let err = $state('');
	let spendErr = $state('');

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let deptId = $derived(page.params.id);

	$effect(() => {
		const id = deptId;
		if (!id) return;
		err = '';
		void getDeptConfig(id)
			.then((c) => {
				config = c;
			})
			.catch((e) => {
				err = e instanceof Error ? e.message : String(e);
			});
	});

	$effect(() => {
		const id = deptId;
		const sid = sessionId;
		if (!id) return;
		spendErr = '';
		void getAnalyticsSpend(id, sid)
			.then((s) => {
				spend = s;
			})
			.catch((e) => {
				spend = null;
				spendErr = e instanceof Error ? e.message : String(e);
			});
	});
</script>

<div class="h-full overflow-auto p-4">
	<h1 class="mb-4 text-lg font-semibold">Department config</h1>
	{#if err}
		<p class="text-sm text-destructive">{err}</p>
	{:else if !config}
		<p class="text-sm text-muted-foreground">Loading…</p>
	{:else}
		{#if spend}
			<section
				class="mb-6 max-w-2xl rounded-lg border border-border bg-card p-4 text-sm"
				aria-label="AI spend"
			>
				<h2 class="mb-2 font-semibold text-foreground">AI spend</h2>
				{#if spend.budget_warning}
					<div
						class="mb-3 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-amber-100"
						role="status"
					>
						<p class="font-medium">Budget warning</p>
						<p class="mt-1 text-xs text-amber-200/90">
							Session spend is at or above 80% of the configured budget.
						</p>
					</div>
				{/if}
				<dl class="grid gap-2 sm:grid-cols-[minmax(0,10rem)_1fr]">
					<dt class="text-muted-foreground">This department (USD)</dt>
					<dd class="font-mono tabular-nums">
						{spend.total_usd.toFixed(4)}
					</dd>
					{#if spend.session_id != null && spend.session_total_usd != null}
						<dt class="text-muted-foreground">Session total (USD)</dt>
						<dd class="font-mono tabular-nums">
							{spend.session_total_usd.toFixed(4)}
						</dd>
					{/if}
					{#if spend.session_budget_limit_usd != null}
						<dt class="text-muted-foreground">Session budget (USD)</dt>
						<dd class="font-mono tabular-nums">
							{spend.session_budget_limit_usd.toFixed(2)}
						</dd>
					{/if}
					{#if spend.budget_usage_ratio != null}
						<dt class="text-muted-foreground">Budget usage</dt>
						<dd class="font-mono tabular-nums">
							{(spend.budget_usage_ratio * 100).toFixed(1)}%
						</dd>
					{/if}
				</dl>
				{#if Object.keys(spend.by_department).length > 0}
					<p class="mt-3 text-xs text-muted-foreground">All departments (USD)</p>
					<ul class="mt-1 max-h-32 overflow-auto font-mono text-xs tabular-nums">
						{#each Object.entries(spend.by_department).sort((a, b) => b[1] - a[1]) as [d, v]}
							<li class="flex justify-between gap-2 py-0.5">
								<span class="text-muted-foreground">{d}</span>
								<span>{v.toFixed(4)}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</section>
		{:else if spendErr}
			<p class="mb-4 text-sm text-destructive">Spend: {spendErr}</p>
		{/if}

		<dl class="grid max-w-2xl gap-3 text-sm">
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Engine</dt>
				<dd class="font-mono text-xs">{config.engine}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Model</dt>
				<dd class="font-mono text-xs">{config.model}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Effort</dt>
				<dd>{config.effort}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Permission mode</dt>
				<dd>{config.permission_mode}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Max budget (USD)</dt>
				<dd>{config.max_budget_usd ?? '—'}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Max turns</dt>
				<dd>{config.max_turns ?? '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Allowed tools</dt>
				<dd class="font-mono text-xs">{config.allowed_tools.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Disallowed tools</dt>
				<dd class="font-mono text-xs">{config.disallowed_tools.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Add dirs</dt>
				<dd class="font-mono text-xs">{config.add_dirs.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">System prompt</dt>
				<dd class="whitespace-pre-wrap rounded-md bg-secondary p-3 font-mono text-xs">{config.system_prompt}</dd>
			</div>
		</dl>
	{/if}
</div>
