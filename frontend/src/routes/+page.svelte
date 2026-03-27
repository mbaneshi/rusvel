<script lang="ts">
	import { onMount } from 'svelte';
	import { activeSession, onboarding, departments } from '$lib/stores';
	import {
		getGoals,
		getEvents,
		getAnalytics,
		getAnalyticsDashboard,
		getVisualReports,
		getBriefLatest
	} from '$lib/api';
	import type {
		Goal,
		Event,
		AnalyticsData,
		AnalyticsSpendResponse,
		DepartmentDef,
		VisualReport,
		ExecutiveBriefRow
	} from '$lib/api';
	import { deptHref, resolveDeptId } from '$lib/api';
	import { toast } from 'svelte-sonner';

	let goals: Goal[] = $state([]);
	let events: Event[] = $state([]);
	let analytics: AnalyticsData | null = $state(null);
	let dashboardSpend: AnalyticsSpendResponse | null = $state(null);
	let visualReport: VisualReport | null = $state(null);
	let latestBrief: ExecutiveBriefRow | null = $state(null);
	let deptList: DepartmentDef[] = $state([]);
	let loading = $state(false);
	let error = $state('');
	let currentSession: import('$lib/api').SessionSummary | null = $state(null);

	departments.subscribe((v) => (deptList = v));
	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) loadData(v.id);
		void loadAnalytics();
	});

	async function loadAnalytics() {
		try {
			if (currentSession) {
				const dash = await getAnalyticsDashboard(currentSession.id);
				analytics = dash;
				dashboardSpend = dash.spend;
			} else {
				analytics = await getAnalytics();
				dashboardSpend = null;
			}
		} catch {
			analytics = null;
			dashboardSpend = null;
		}
	}

	onMount(async () => {
		await loadAnalytics();
		try {
			const reports = await getVisualReports();
			if (reports.length > 0) visualReport = reports[reports.length - 1];
		} catch {
			/* visual reports optional */
		}
	});

	async function loadData(sessionId: string) {
		loading = true;
		error = '';
		try {
			const [goalsResult, eventsResult, briefResult] = await Promise.allSettled([
				getGoals(sessionId),
				getEvents(sessionId),
				getBriefLatest(sessionId)
			]);
			goals = goalsResult.status === 'fulfilled' ? goalsResult.value : [];
			events = eventsResult.status === 'fulfilled' ? eventsResult.value : [];
			latestBrief = briefResult.status === 'fulfilled' ? briefResult.value : null;
			if (goals.length > 0) onboarding.complete('goalAdded');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
			toast.error(error);
		} finally {
			loading = false;
		}
	}

	function formatTime(iso: string): string {
		try {
			return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		} catch {
			return iso;
		}
	}

	// Compute event counts by source for the mini bar chart
	let eventsBySource = $derived(() => {
		const counts: Record<string, number> = {};
		for (const e of events) {
			counts[e.source] = (counts[e.source] || 0) + 1;
		}
		return Object.entries(counts)
			.sort((a, b) => b[1] - a[1])
			.slice(0, 6);
	});

	let maxEventCount = $derived(() => {
		const entries = eventsBySource();
		return entries.length > 0 ? Math.max(...entries.map((e) => e[1])) : 1;
	});

	let planDeptHref = $derived(deptHref(resolveDeptId(deptList, 'forge', 'forge')));
</script>

<div class="h-full overflow-y-auto p-6">
	<h1 class="mb-6 text-2xl font-bold text-foreground">Dashboard</h1>

	{#if !currentSession}
		<div
			class="flex flex-col items-center justify-center rounded-xl border border-border bg-card py-16 px-8"
		>
			<div
				class="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-primary/30 to-chart-4/20"
			>
				<span class="text-3xl font-bold text-primary">R</span>
			</div>
			<h2 class="text-xl font-semibold text-foreground">Welcome to RUSVEL</h2>
			<p class="mt-2 max-w-md text-center text-sm text-muted-foreground">
				Your AI-powered virtual agency. Create a session to start planning your day, managing goals,
				and chatting with department agents.
			</p>
			<div class="mt-6 flex flex-col items-center gap-3">
				<p class="text-xs text-muted-foreground/60">
					Click <strong class="text-muted-foreground">+ New Session</strong> in the sidebar to begin
				</p>
				<div class="flex items-center gap-2 text-[10px] text-muted-foreground/60">
					<span>or press</span>
					<kbd class="rounded border border-border bg-secondary px-1.5 py-0.5 text-muted-foreground"
						>⌘K</kbd
					>
					<span>to open the command palette</span>
				</div>
			</div>
		</div>
	{:else if loading}
		<div class="flex items-center gap-3 text-muted-foreground">
			<div class="h-5 w-5 animate-spin rounded-full border-2 border-border border-t-primary"></div>
			Loading...
		</div>
	{:else if error}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 p-4 text-destructive">
			{error}
		</div>
	{:else}
		<!-- Session Header -->
		<div class="mb-6 flex items-center gap-3">
			<span class="rounded-md bg-primary/15 px-2 py-0.5 text-xs font-medium text-primary">
				{currentSession.kind}
			</span>
			<h2 class="text-lg font-semibold text-foreground">{currentSession.name}</h2>
		</div>

		{#if latestBrief}
			<div class="mb-6 rounded-xl border border-border bg-card p-4">
				<div class="flex items-start justify-between gap-3">
					<div>
						<p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
							Latest executive brief
						</p>
						<p class="mt-1 text-sm text-foreground line-clamp-3">{latestBrief.summary}</p>
						<p class="mt-2 text-[10px] text-muted-foreground">
							{latestBrief.date} &middot;
							<a href={planDeptHref} class="text-primary hover:text-primary/80">Forge</a>
						</p>
					</div>
				</div>
			</div>
		{/if}

		<!-- Analytics Overview (agency-wide) -->
		{#if analytics}
			<div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-5">
				<div class="rounded-xl border border-border bg-card p-4">
					<p class="text-xs font-medium text-muted-foreground">Agents</p>
					<p class="mt-1 text-2xl font-bold text-chart-1">{analytics.agents}</p>
				</div>
				<div class="rounded-xl border border-border bg-card p-4">
					<p class="text-xs font-medium text-muted-foreground">Skills</p>
					<p class="mt-1 text-2xl font-bold text-chart-2">{analytics.skills}</p>
				</div>
				<div class="rounded-xl border border-border bg-card p-4">
					<p class="text-xs font-medium text-muted-foreground">Rules</p>
					<p class="mt-1 text-2xl font-bold text-chart-3">{analytics.rules}</p>
				</div>
				<div class="rounded-xl border border-border bg-card p-4">
					<p class="text-xs font-medium text-muted-foreground">Conversations</p>
					<p class="mt-1 text-2xl font-bold text-chart-4">{analytics.conversations}</p>
				</div>
				{#if dashboardSpend}
					<div class="rounded-xl border border-border bg-card p-4 sm:col-span-2 lg:col-span-1">
						<p class="text-xs font-medium text-muted-foreground">LLM spend (session)</p>
						<p class="mt-1 text-2xl font-bold text-primary">
							${(dashboardSpend.session_total_usd ?? dashboardSpend.total_usd).toFixed(4)}
						</p>
						<p class="mt-1 text-[10px] text-muted-foreground">
							<a href="/settings/spend" class="text-primary hover:underline">Details →</a>
						</p>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Visual Health -->
		{#if visualReport}
			{@const s = visualReport.summary}
			{@const statusColor =
				s.critical > 0
					? 'text-destructive'
					: s.high > 0
						? 'text-orange-500'
						: s.regressions > 0
							? 'text-yellow-500'
							: 'text-green-500'}
			{@const statusLabel =
				s.critical > 0
					? 'Critical'
					: s.high > 0
						? 'Issues'
						: s.regressions > 0
							? 'Minor'
							: 'Healthy'}
			<div class="mb-6 rounded-xl border border-border bg-card p-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div
							class="flex h-8 w-8 items-center justify-center rounded-lg bg-secondary"
						>
							<svg
								class="h-4 w-4 {statusColor}"
								viewBox="0 0 16 16"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
							>
								<rect x="2" y="2" width="5" height="5" rx="1" />
								<rect x="9" y="2" width="5" height="5" rx="1" />
								<rect x="2" y="9" width="5" height="5" rx="1" />
								<rect x="9" y="9" width="5" height="5" rx="1" />
							</svg>
						</div>
						<div>
							<p class="text-sm font-medium text-foreground">
								Visual Health: <span class={statusColor}>{statusLabel}</span>
							</p>
							<p class="text-xs text-muted-foreground">
								{s.total_routes} routes tested &middot; {s.regressions} regressions
								{#if s.critical > 0}<span class="text-destructive"
										>({s.critical} critical)</span
									>{/if}
							</p>
						</div>
					</div>
					<span class="text-xs text-muted-foreground">
						{new Date(visualReport.timestamp).toLocaleDateString()}
					</span>
				</div>
			</div>
		{/if}

		<!-- Session Stats Row -->
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">Goals</p>
				<p class="mt-1 text-2xl font-bold text-foreground">{goals.length}</p>
				<p class="text-xs text-muted-foreground">
					{goals.filter((g) => g.status === 'Active').length} active
				</p>
			</div>
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">Events</p>
				<p class="mt-1 text-2xl font-bold text-foreground">{events.length}</p>
				<p class="text-xs text-muted-foreground">total logged</p>
			</div>
			<div class="rounded-xl border border-border bg-card p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
					Departments
				</p>
				<p class="mt-1 text-2xl font-bold text-primary">{deptList.length}</p>
				<p class="text-xs text-muted-foreground">
					<a href={planDeptHref} class="text-primary hover:text-primary/80">Generate plan &rarr;</a>
				</p>
			</div>
		</div>

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Active Goals -->
			<div class="rounded-xl border border-border bg-card p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					Active Goals
				</h3>
				{#if goals.length === 0}
					<div class="flex flex-col items-center py-6 text-center">
						<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-primary/15">
							<svg
								class="h-5 w-5 text-primary"
								viewBox="0 0 16 16"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"><path d="M3 8l3.5 3.5L13 4" /></svg
							>
						</div>
						<p class="text-sm text-muted-foreground">No goals yet</p>
						<p class="mt-1 text-xs text-muted-foreground/60">
							Goals help you stay focused and track progress
						</p>
						<a
							href={planDeptHref}
							class="mt-3 rounded-lg bg-primary px-4 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90"
						>
							Add your first goal
						</a>
					</div>
				{:else}
					<ul class="space-y-2">
						{#each goals as goal}
							<li class="rounded-lg bg-secondary/50 p-3">
								<div class="mb-1 flex items-center justify-between">
									<p class="text-sm font-medium text-foreground">{goal.title}</p>
									<span class="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground"
										>{goal.timeframe}</span
									>
								</div>
								<div class="flex items-center gap-2">
									<div class="h-1.5 flex-1 rounded-full bg-secondary">
										<div
											class="h-1.5 rounded-full bg-primary transition-all duration-500"
											style="width: {Math.round(goal.progress * 100)}%"
										></div>
									</div>
									<span class="text-xs text-muted-foreground"
										>{Math.round(goal.progress * 100)}%</span
									>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<!-- Event Activity by Source (mini bar chart) -->
			<div class="rounded-xl border border-border bg-card p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					Activity by Engine
				</h3>
				{#if events.length === 0}
					<div class="flex flex-col items-center py-6 text-center">
						<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-chart-2/15">
							<svg
								class="h-5 w-5 text-chart-2"
								viewBox="0 0 16 16"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
								><path d="M8 3.5V8L10.5 10.5" stroke-linecap="round" /><circle
									cx="8"
									cy="8"
									r="5.5"
								/></svg
							>
						</div>
						<p class="text-sm text-muted-foreground">No events yet</p>
						<p class="mt-1 text-xs text-muted-foreground/60">
							Events are logged when you generate plans, add goals, or chat with departments
						</p>
					</div>
				{:else}
					<div class="space-y-2">
						{#each eventsBySource() as [source, count]}
							{@const pct = (count / maxEventCount()) * 100}
							{@const colors = [
								'bg-chart-1',
								'bg-chart-2',
								'bg-chart-3',
								'bg-chart-4',
								'bg-chart-5',
								'bg-primary'
							]}
							{@const colorIdx = eventsBySource().findIndex((e) => e[0] === source)}
							<div class="flex items-center gap-3">
								<span class="w-16 text-xs font-mono text-muted-foreground truncate">{source}</span>
								<div class="flex-1 h-5 rounded-md bg-secondary overflow-hidden">
									<div
										class="h-full rounded-md {colors[
											colorIdx % colors.length
										]} transition-all duration-500"
										style="width: {pct}%"
									></div>
								</div>
								<span class="w-8 text-right text-xs font-medium text-foreground">{count}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Recent Events -->
			<div class="rounded-xl border border-border bg-card p-5 lg:col-span-2">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
					Recent Events
				</h3>
				{#if events.length === 0}
					<p class="text-sm text-muted-foreground">No events yet.</p>
				{:else}
					<div class="space-y-1">
						{#each events.slice(0, 15) as event}
							<div class="flex items-center gap-3 rounded-md px-2 py-1.5 hover:bg-secondary/50">
								<span
									class="rounded bg-secondary px-1.5 py-0.5 text-xs font-mono text-muted-foreground"
									>{event.source}</span
								>
								<span class="flex-1 text-sm text-foreground/80 truncate">{event.kind}</span>
								<span class="text-xs text-muted-foreground">{formatTime(event.created_at)}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
