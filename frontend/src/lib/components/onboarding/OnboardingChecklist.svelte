<script lang="ts">
	import { onboarding, sessions, departments } from '$lib/stores';
	import type { OnboardingState } from '$lib/stores';
	import { goto } from '$app/navigation';
	import { deptHref, resolveDeptId } from '$lib/api';
	import type { DepartmentDef } from '$lib/api';

	let ob: OnboardingState = $state({
		sessionCreated: false,
		goalAdded: false,
		planGenerated: false,
		deptChatUsed: false,
		agentCreated: false,
		dismissed: false,
		tourCompleted: false
	});
	let expanded = $state(true);

	onboarding.subscribe((v) => (ob = v));

	// Auto-detect session creation
	sessions.subscribe((list) => {
		if (list.length > 0 && !ob.sessionCreated) {
			onboarding.complete('sessionCreated');
		}
	});

	let deptList: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (deptList = v));

	const steps = $derived.by(() => {
		const forgeId = resolveDeptId(deptList, 'forge', 'forge');
		const codeId = resolveDeptId(deptList, 'code', 'code');
		return [
			{
				key: 'sessionCreated' as const,
				label: 'Create your first session',
				action: () => {
					/* sidebar handles this */
				}
			},
			{
				key: 'goalAdded' as const,
				label: 'Add a goal',
				action: () => goto(deptHref(forgeId))
			},
			{
				key: 'planGenerated' as const,
				label: 'Generate a daily plan',
				action: () => goto(deptHref(forgeId))
			},
			{
				key: 'deptChatUsed' as const,
				label: 'Chat with a department',
				action: () => goto(deptHref(codeId))
			},
			{
				key: 'agentCreated' as const,
				label: 'Create an agent',
				action: () => goto(deptHref(forgeId))
			}
		];
	});

	let completedCount = $derived(steps.filter((s) => ob[s.key]).length);
	let allDone = $derived(completedCount === steps.length);
</script>

{#if !ob.dismissed && !allDone}
	<div class="fixed bottom-4 left-16 z-30 w-72 rounded-xl border border-border bg-card shadow-2xl">
		<!-- Header -->
		<button
			onclick={() => (expanded = !expanded)}
			class="flex w-full items-center justify-between px-4 py-3 text-left"
		>
			<div class="flex items-center gap-2">
				<div
					class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-[10px] font-bold text-primary-foreground"
				>
					{completedCount}
				</div>
				<span class="text-sm font-medium text-foreground">Getting Started</span>
			</div>
			<div class="flex items-center gap-1">
				<span class="text-[10px] text-muted-foreground">{completedCount}/{steps.length}</span>
				<svg
					class="h-3.5 w-3.5 text-muted-foreground transition-transform {expanded
						? 'rotate-180'
						: ''}"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"><path d="M4 6l4 4 4-4" /></svg
				>
			</div>
		</button>

		<!-- Progress bar -->
		<div class="mx-4 h-1 rounded-full bg-secondary">
			<div
				class="h-1 rounded-full bg-primary transition-all duration-500"
				style="width: {(completedCount / steps.length) * 100}%"
			></div>
		</div>

		{#if expanded}
			<!-- Steps -->
			<div class="px-4 py-3 space-y-2">
				{#each steps as step}
					<button
						onclick={step.action}
						disabled={ob[step.key]}
						class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-xs transition-colors
							{ob[step.key] ? 'text-muted-foreground line-through' : 'text-foreground hover:bg-secondary'}"
					>
						{#if ob[step.key]}
							<div
								class="flex h-4 w-4 flex-shrink-0 items-center justify-center rounded-full bg-chart-2/30"
							>
								<svg class="h-2.5 w-2.5 text-chart-2" viewBox="0 0 12 12" fill="currentColor"
									><path
										d="M10.28 2.28a.75.75 0 00-1.06-1.06L4.5 5.94 2.78 4.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.06 0l5.25-5.25z"
									/></svg
								>
							</div>
						{:else}
							<div class="h-4 w-4 flex-shrink-0 rounded-full border border-border"></div>
						{/if}
						{step.label}
					</button>
				{/each}
			</div>

			<!-- Dismiss -->
			<div class="border-t border-border px-4 py-2">
				<button
					onclick={() => onboarding.dismiss()}
					class="text-[10px] text-muted-foreground hover:text-foreground"
				>
					Dismiss
				</button>
			</div>
		{/if}
	</div>
{/if}
