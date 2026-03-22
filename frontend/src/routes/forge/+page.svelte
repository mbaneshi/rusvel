<script lang="ts">
	import { activeSession } from '$lib/stores';
	import { getGoals, createGoal, getMissionToday } from '$lib/api';
	import type { Goal, DailyPlan } from '$lib/api';

	let goals: Goal[] = $state([]);
	let plan: DailyPlan | null = $state(null);
	let loading = $state(false);
	let planLoading = $state(false);
	let error = $state('');

	let showAddGoal = $state(false);
	let goalTitle = $state('');
	let goalDescription = $state('');
	let goalTimeframe = $state('Week');

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) loadGoals(v.id);
	});

	async function loadGoals(sessionId: string) {
		loading = true;
		error = '';
		try {
			goals = await getGoals(sessionId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load goals';
		} finally {
			loading = false;
		}
	}

	async function handleAddGoal() {
		if (!currentSession || !goalTitle.trim()) return;
		try {
			const goal = await createGoal(currentSession.id, {
				title: goalTitle.trim(),
				description: goalDescription.trim(),
				timeframe: goalTimeframe
			});
			goals = [...goals, goal];
			goalTitle = '';
			goalDescription = '';
			showAddGoal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create goal';
		}
	}

	async function generatePlan() {
		if (!currentSession) return;
		planLoading = true;
		error = '';
		try {
			plan = await getMissionToday(currentSession.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to generate plan';
		} finally {
			planLoading = false;
		}
	}

	function statusColor(status: string): string {
		switch (status) {
			case 'Active':
				return 'bg-green-900/50 text-green-300';
			case 'Completed':
				return 'bg-blue-900/50 text-blue-300';
			case 'Abandoned':
				return 'bg-red-900/50 text-red-300';
			case 'Deferred':
				return 'bg-yellow-900/50 text-yellow-300';
			default:
				return 'bg-gray-800 text-gray-400';
		}
	}

	function progressWidth(progress: number): string {
		return `${Math.round(progress * 100)}%`;
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-bold text-gray-100">Forge / Mission</h1>
		{#if currentSession}
			<button
				onclick={generatePlan}
				disabled={planLoading}
				class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium hover:bg-indigo-500 disabled:opacity-50"
			>
				{planLoading ? 'Generating...' : 'Generate Daily Plan'}
			</button>
		{/if}
	</div>

	{#if !currentSession}
		<div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
			<p class="text-lg text-gray-400">No session selected</p>
			<p class="mt-2 text-sm text-gray-600">Select a session from the sidebar.</p>
		</div>
	{:else if error}
		<div class="mb-4 rounded-xl border border-red-900 bg-red-950 p-4 text-red-400">{error}</div>
	{/if}

	{#if currentSession}
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Goals -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<div class="mb-4 flex items-center justify-between">
					<h3 class="text-sm font-semibold uppercase tracking-wider text-gray-400">Goals</h3>
					<button
						onclick={() => (showAddGoal = !showAddGoal)}
						class="rounded-md bg-gray-800 px-3 py-1 text-xs text-gray-400 hover:bg-gray-700 hover:text-gray-200"
					>
						+ Add Goal
					</button>
				</div>

				{#if showAddGoal}
					<div class="mb-4 space-y-2 rounded-lg bg-gray-800/50 p-3">
						<input
							bind:value={goalTitle}
							placeholder="Goal title"
							class="w-full rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 focus:border-indigo-500 focus:outline-none"
						/>
						<textarea
							bind:value={goalDescription}
							placeholder="Description"
							rows="2"
							class="w-full rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 focus:border-indigo-500 focus:outline-none"
						></textarea>
						<div class="flex gap-2">
							<select
								bind:value={goalTimeframe}
								class="flex-1 rounded-md border border-gray-700 bg-gray-800 px-2 py-1.5 text-sm text-gray-200"
							>
								<option>Day</option>
								<option>Week</option>
								<option>Month</option>
								<option>Quarter</option>
							</select>
							<button
								onclick={handleAddGoal}
								class="rounded-md bg-indigo-600 px-4 py-1.5 text-sm font-medium hover:bg-indigo-500"
							>
								Save
							</button>
						</div>
					</div>
				{/if}

				{#if loading}
					<div class="flex items-center gap-2 text-gray-400">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-gray-600 border-t-indigo-500"
						></div>
						Loading...
					</div>
				{:else if goals.length === 0}
					<p class="text-sm text-gray-500">No goals set. Add one to get started.</p>
				{:else}
					<ul class="space-y-3">
						{#each goals as goal}
							<li class="rounded-lg bg-gray-800/50 p-3">
								<div class="mb-1 flex items-center justify-between">
									<h4 class="text-sm font-medium text-gray-200">{goal.title}</h4>
									<span class={`rounded-full px-2 py-0.5 text-xs ${statusColor(goal.status)}`}>
										{goal.status}
									</span>
								</div>
								<p class="mb-2 text-xs text-gray-400">{goal.description}</p>
								<div class="flex items-center gap-2">
									<span class="text-xs text-gray-500">{goal.timeframe}</span>
									<div class="h-1.5 flex-1 rounded-full bg-gray-700">
										<div
											class="h-1.5 rounded-full bg-indigo-500"
											style="width: {progressWidth(goal.progress)}"
										></div>
									</div>
									<span class="text-xs text-gray-500"
										>{Math.round(goal.progress * 100)}%</span
									>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<!-- Today's Plan -->
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">
					Today's Plan
				</h3>

				{#if planLoading}
					<div class="flex items-center gap-2 text-gray-400">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-gray-600 border-t-indigo-500"
						></div>
						Generating plan...
					</div>
				{:else if plan}
					{#if plan.focus_areas.length > 0}
						<div class="mb-3 flex flex-wrap gap-2">
							{#each plan.focus_areas as area}
								<span class="rounded-full bg-indigo-900/30 px-2 py-0.5 text-xs text-indigo-300"
									>{area}</span
								>
							{/each}
						</div>
					{/if}
					<ul class="space-y-2">
						{#each plan.tasks as task}
							<li class="flex items-start gap-3 rounded-lg bg-gray-800/50 p-3">
								<input type="checkbox" class="mt-1 accent-indigo-500" />
								<div class="flex-1">
									<p class="text-sm text-gray-200">{task.title}</p>
									<span class="text-xs text-gray-500">{task.priority}</span>
								</div>
							</li>
						{/each}
					</ul>
					{#if plan.notes}
						<div class="mt-3 rounded-lg bg-gray-800/30 p-2 text-xs text-gray-500">
							{plan.notes}
						</div>
					{/if}
				{:else}
					<p class="text-sm text-gray-500">
						Click "Generate Daily Plan" to create a prioritized task list from your goals.
					</p>
				{/if}
			</div>
		</div>
	{/if}
</div>
