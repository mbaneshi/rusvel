<script lang="ts">
	import { onMount } from 'svelte';
	import { activeSession } from '$lib/stores';
	import { getMissionToday, getEvents } from '$lib/api';
	import type { DailyPlan, Event } from '$lib/api';

	let plan: DailyPlan | null = $state(null);
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
			const [planResult, eventsResult] = await Promise.allSettled([
				getMissionToday(sessionId),
				getEvents(sessionId)
			]);
			plan = planResult.status === 'fulfilled' ? planResult.value : null;
			events = eventsResult.status === 'fulfilled' ? eventsResult.value : [];
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	function priorityColor(priority: string): string {
		switch (priority) {
			case 'Urgent':
				return 'text-red-400';
			case 'High':
				return 'text-orange-400';
			case 'Medium':
				return 'text-yellow-400';
			case 'Low':
				return 'text-gray-400';
			default:
				return 'text-gray-400';
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

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Today's Plan -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">
					Today's Plan
				</h3>
				{#if plan}
					{#if plan.focus_areas.length > 0}
						<div class="mb-3 flex flex-wrap gap-2">
							{#each plan.focus_areas as area}
								<span class="rounded-full bg-gray-800 px-2 py-0.5 text-xs text-gray-300"
									>{area}</span
								>
							{/each}
						</div>
					{/if}
					<ul class="space-y-2">
						{#each plan.tasks as task}
							<li class="flex items-start gap-3 rounded-lg bg-gray-800/50 p-3">
								<span class={`mt-0.5 text-xs font-medium ${priorityColor(task.priority)}`}>
									{task.priority}
								</span>
								<div>
									<p class="text-sm text-gray-200">{task.title}</p>
									<p class="text-xs text-gray-500">{task.status}</p>
								</div>
							</li>
						{/each}
					</ul>
					{#if plan.notes}
						<p class="mt-3 text-xs text-gray-500">{plan.notes}</p>
					{/if}
				{:else}
					<p class="text-sm text-gray-500">
						No plan generated yet. Visit Forge to create one.
					</p>
				{/if}
			</div>

			<!-- Recent Events -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">
					Recent Events
				</h3>
				{#if events.length === 0}
					<p class="text-sm text-gray-500">No events yet.</p>
				{:else}
					<ul class="space-y-2">
						{#each events.slice(0, 20) as event}
							<li class="flex items-start gap-3 border-l-2 border-gray-700 py-1 pl-3">
								<div class="flex-1">
									<div class="flex items-center gap-2">
										<span
											class="rounded bg-gray-800 px-1.5 py-0.5 text-xs font-mono text-gray-400"
											>{event.source}</span
										>
										<span class="text-xs text-gray-500"
											>{formatTime(event.created_at)}</span
										>
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
