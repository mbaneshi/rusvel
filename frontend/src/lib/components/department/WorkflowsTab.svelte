<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { Confetti } from 'svelte-confetti';
	import { getWorkflows, createWorkflow, deleteWorkflow, runWorkflow, getAgents } from '$lib/api';
	import type { Workflow, WorkflowStepDef, WorkflowRunResult, Agent } from '$lib/api';
	import WorkflowBuilder from '$lib/components/workflow/WorkflowBuilder.svelte';

	let { dept, deptHsl }: { dept: string; deptHsl: string } = $props();

	let workflows: Workflow[] = $state([]);
	let agents: Agent[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newDesc = $state('');
	let newSteps: WorkflowStepDef[] = $state([]);
	let runningWorkflowId: string | null = $state(null);
	let workflowResults: WorkflowRunResult | null = $state(null);
	let showConfetti = $state(false);

	onMount(() => {
		loadWorkflows();
		loadAgents();
	});

	async function loadWorkflows() {
		try {
			workflows = await getWorkflows();
		} catch {
			workflows = [];
		}
	}

	async function loadAgents() {
		try {
			agents = await getAgents(dept);
		} catch {
			agents = [];
		}
	}

	async function handleCreate() {
		if (!newName.trim() || newSteps.length === 0) return;
		try {
			await createWorkflow({
				name: newName.trim(),
				description: newDesc,
				steps: newSteps,
				metadata: { engine: dept }
			});
			newName = '';
			newDesc = '';
			newSteps = [];
			showCreate = false;
			await loadWorkflows();
			toast.success('Workflow created');
		} catch (e) {
			toast.error(`Failed to create workflow: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteWorkflow(id);
			await loadWorkflows();
			toast.success('Workflow deleted');
		} catch (e) {
			toast.error(`Failed to delete workflow: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleRun(id: string) {
		runningWorkflowId = id;
		workflowResults = null;
		try {
			workflowResults = await runWorkflow(id);
			toast.success(`Workflow completed ($${workflowResults.total_cost_usd.toFixed(4)})`);
			showConfetti = true;
			setTimeout(() => (showConfetti = false), 3000);
		} catch (e: unknown) {
			workflowResults = null;
			const msg = e instanceof Error ? e.message : String(e);
			toast.error(`Workflow failed: ${msg}`);
		} finally {
			runningWorkflowId = null;
		}
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:border-[hsl({deptHsl}/.3)] hover:text-foreground"
	>
		+ New Workflow
	</button>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Workflow name"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<input
				bind:value={newDesc}
				placeholder="Description (optional)"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>

			<div class="border-t border-border pt-2">
				<p class="text-[10px] font-medium text-muted-foreground mb-1">Steps ({newSteps.length})</p>
				<WorkflowBuilder
					bind:steps={newSteps}
					agents={agents.map((a) => ({ name: a.name, role: a.role }))}
				/>
			</div>

			<button
				onclick={handleCreate}
				disabled={!newName.trim() || newSteps.length === 0}
				class="w-full rounded-md py-1 text-xs font-medium text-white disabled:opacity-40 disabled:cursor-not-allowed"
				style="background-color: hsl({deptHsl})">Create Workflow</button
			>
		</div>
	{/if}

	{#each workflows as wf}
		<div class="rounded-lg bg-secondary p-2.5 group">
			<div class="flex items-center justify-between mb-1">
				<span class="text-xs font-medium text-foreground">{wf.name}</span>
				<div class="flex items-center gap-1">
					<span
						class="rounded px-1.5 py-0.5 text-[9px]"
						style="background-color: hsl({deptHsl}/.15); color: hsl({deptHsl})"
						>{wf.steps.length} steps</span
					>
					<button
						onclick={() => handleDelete(wf.id)}
						class="hidden group-hover:block text-muted-foreground hover:text-destructive text-[10px]"
						>x</button
					>
				</div>
			</div>
			{#if wf.description}
				<p class="text-[10px] text-muted-foreground mb-1">{wf.description}</p>
			{/if}
			<div class="space-y-0.5 mb-2">
				{#each wf.steps as step, i}
					<div class="flex items-center gap-1 text-[9px] text-muted-foreground">
						<span>{i + 1}.</span>
						<span class="font-mono" style="color: hsl({deptHsl})">@{step.agent_name}</span>
						<span class="truncate"
							>{step.prompt_template.slice(0, 25)}{step.prompt_template.length > 25
								? '...'
								: ''}</span
						>
					</div>
				{/each}
			</div>
			<button
				onclick={() => handleRun(wf.id)}
				disabled={runningWorkflowId === wf.id}
				class="w-full rounded-md py-1 text-[10px] font-medium text-white disabled:opacity-50"
				style="background-color: hsl({deptHsl}/.8)"
			>
				{runningWorkflowId === wf.id ? 'Running...' : 'Run Workflow'}
			</button>
		</div>
	{/each}

	{#if workflows.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">
			No workflows. Create one to chain agents together.
		</p>
	{/if}

	<!-- Workflow Results -->
	{#if workflowResults}
		<div
			class="mt-3 rounded-lg border bg-secondary p-3 space-y-2"
			style="border-color: hsl({deptHsl}/.3)"
		>
			<div class="flex items-center justify-between">
				{#if showConfetti}<Confetti />{/if}
				<span class="text-xs font-medium" style="color: hsl({deptHsl})"
					>Results: {workflowResults.workflow_name}</span
				>
				<span class="text-[9px] text-muted-foreground"
					>${workflowResults.total_cost_usd.toFixed(4)}</span
				>
			</div>
			{#each workflowResults.steps as result}
				<div class="rounded bg-card p-2">
					<div class="flex items-center gap-1 mb-1">
						<span class="text-[9px] text-muted-foreground">Step {result.step_index + 1}</span>
						<span class="text-[10px] font-mono" style="color: hsl({deptHsl})"
							>@{result.agent_name}</span
						>
						<span class="text-[9px] text-muted-foreground ml-auto"
							>${result.cost_usd.toFixed(4)}</span
						>
					</div>
					<p class="text-[10px] text-foreground whitespace-pre-wrap max-h-32 overflow-y-auto">
						{result.output}
					</p>
				</div>
			{/each}
			<button
				onclick={() => (workflowResults = null)}
				class="w-full rounded-md bg-card py-1 text-[10px] text-muted-foreground hover:text-foreground"
				>Dismiss</button
			>
		</div>
	{/if}
</div>
