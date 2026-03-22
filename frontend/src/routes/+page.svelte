<script lang="ts">
	import { onMount } from 'svelte';
	import { activeSession } from '$lib/stores';
	import { getGoals, getEvents } from '$lib/api';
	import type { Goal, Event } from '$lib/api';

	let goals: Goal[] = $state([]);
	let events: Event[] = $state([]);
	let loading = $state(false);
	let error = $state('');
	let currentSession: import('$lib/api').SessionSummary | null = $state(null);

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) loadData(v.id);
	});

	async function loadData(sessionId: string) {
		loading = true;
		error = '';
		try {
			// Load goals and events (both cheap reads from DB).
			// Plan generation is expensive (calls LLM) — only on-demand from Forge page.
			const [goalsResult, eventsResult] = await Promise.allSettled([
				getGoals(sessionId),
				getEvents(sessionId)
			]);
			goals = goalsResult.status === 'fulfilled' ? goalsResult.value : [];
			events = eventsResult.status === 'fulfilled' ? eventsResult.value : [];
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
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
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold text-gray-100">Dashboard</h1>

	{#if !currentSession}
		<div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
			<p class="text-lg text-gray-400">No session selected</p>
			<p class="mt-2 text-sm text-gray-600">Create or select a session from the sidebar to begin.</p>
		</div>
	{:else if loading}
		<div class="flex items-center gap-3 text-gray-400">
			<div
				class="h-5 w-5 animate-spin rounded-full border-2 border-gray-600 border-t-indigo-500"
			></div>
			Loading...
		</div>
	{:else if error}
		<div class="rounded-xl border border-red-900 bg-red-950 p-4 text-red-400">{error}</div>
	{:else}
		<!-- Session Header -->
		<div class="mb-6 flex items-center gap-3">
			<span
				class="rounded-md bg-indigo-900/50 px-2 py-0.5 text-xs font-medium text-indigo-300"
			>
				{currentSession.kind}
			</span>
			<h2 class="text-lg font-semibold text-gray-200">{currentSession.name}</h2>
		</div>

		<!-- Stats Row -->
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-gray-500">Goals</p>
				<p class="mt-1 text-2xl font-bold text-gray-100">{goals.length}</p>
				<p class="text-xs text-gray-500">{goals.filter(g => g.status === 'Active').length} active</p>
			</div>
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-gray-500">Events</p>
				<p class="mt-1 text-2xl font-bold text-gray-100">{events.length}</p>
				<p class="text-xs text-gray-500">total logged</p>
			</div>
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-4">
				<p class="text-xs font-medium uppercase tracking-wider text-gray-500">Engine</p>
				<p class="mt-1 text-2xl font-bold text-indigo-400">Forge</p>
				<p class="text-xs text-gray-500">
					<a href="/forge" class="text-indigo-400 hover:text-indigo-300">Generate plan &rarr;</a>
				</p>
			</div>
		</div>

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Active Goals -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">
					Active Goals
				</h3>
				{#if goals.length === 0}
					<p class="text-sm text-gray-500">
						No goals yet. <a href="/forge" class="text-indigo-400 hover:text-indigo-300">Add goals in Forge</a>
					</p>
				{:else}
					<ul class="space-y-2">
						{#each goals as goal}
							<li class="rounded-lg bg-gray-800/50 p-3">
								<div class="mb-1 flex items-center justify-between">
									<p class="text-sm font-medium text-gray-200">{goal.title}</p>
									<span class="rounded-full bg-gray-700 px-2 py-0.5 text-xs text-gray-400">{goal.timeframe}</span>
								</div>
								<div class="flex items-center gap-2">
									<div class="h-1.5 flex-1 rounded-full bg-gray-700">
										<div class="h-1.5 rounded-full bg-indigo-500" style="width: {Math.round(goal.progress * 100)}%"></div>
									</div>
									<span class="text-xs text-gray-500">{Math.round(goal.progress * 100)}%</span>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<!-- Recent Events -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">
					Recent Events
				</h3>
				{#if events.length === 0}
					<p class="text-sm text-gray-500">No events yet. Actions you take will show up here.</p>
				{:else}
					<ul class="space-y-2">
						{#each events.slice(0, 20) as event}
							<li class="flex items-start gap-3 border-l-2 border-gray-700 py-1 pl-3">
								<div class="flex-1">
									<div class="flex items-center gap-2">
										<span class="rounded bg-gray-800 px-1.5 py-0.5 text-xs font-mono text-gray-400">{event.source}</span>
										<span class="text-xs text-gray-500">{formatTime(event.created_at)}</span>
									</div>
									<p class="mt-0.5 text-sm text-gray-300">{event.kind}</p>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</div>
	{/if}
</div>
