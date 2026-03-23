<script lang="ts">
	import {
		SvelteFlow,
		Controls,
		Background,
		MiniMap,
		type Node,
		type Edge,
		type NodeTypes
	} from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import AgentNode from './AgentNode.svelte';
	import type { WorkflowStepDef } from '$lib/api';

	let {
		steps = $bindable([]),
		agents = [],
		onchange
	}: {
		steps?: WorkflowStepDef[];
		agents?: { name: string; role?: string }[];
		onchange?: (steps: WorkflowStepDef[]) => void;
	} = $props();

	const nodeTypes: NodeTypes = {
		agent: AgentNode as any
	};

	// Convert steps to nodes + edges
	let nodes: Node[] = $state([]);
	let edges: Edge[] = $state([]);

	function stepsToGraph(stepList: WorkflowStepDef[]) {
		nodes = stepList.map((step, i) => ({
			id: `step-${i}`,
			type: 'agent',
			position: { x: 50, y: i * 120 + 30 },
			data: { label: step.agent_name, prompt: step.prompt_template, index: i }
		}));
		edges = stepList.slice(1).map((_, i) => ({
			id: `edge-${i}`,
			source: `step-${i}`,
			target: `step-${i + 1}`,
			animated: true,
			style: 'stroke: oklch(0.55 0.18 270); stroke-width: 2px;'
		}));
	}

	$effect(() => {
		stepsToGraph(steps);
	});

	function addStep() {
		const name = agents.length > 0 ? agents[0].name : 'New Agent';
		steps = [
			...steps,
			{ agent_name: name, prompt_template: 'Do the task', step_type: 'sequential' }
		];
		onchange?.(steps);
	}

	function removeStep(index: number) {
		steps = steps.filter((_, i) => i !== index);
		onchange?.(steps);
	}

	function updateStep(index: number, field: 'agent_name' | 'prompt_template', value: string) {
		steps = steps.map((s, i) => (i === index ? { ...s, [field]: value } : s));
		onchange?.(steps);
	}
</script>

<div class="flex flex-col gap-2">
	<!-- Visual flow canvas -->
	{#if steps.length > 0}
		<div class="h-48 rounded-lg border border-border overflow-hidden bg-background">
			<SvelteFlow
				{nodes}
				{edges}
				{nodeTypes}
				fitView
				nodesDraggable={false}
				nodesConnectable={false}
				elementsSelectable={false}
				panOnDrag={false}
				zoomOnScroll={false}
				preventScrolling={false}
				proOptions={{ hideAttribution: true }}
			>
				<Background gap={16} class="!bg-background" />
			</SvelteFlow>
		</div>
	{/if}

	<!-- Step list (editable) -->
	<div class="space-y-1.5">
		{#each steps as step, i}
			<div class="flex items-center gap-1.5 rounded-md bg-secondary/50 px-2 py-1.5">
				<span
					class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/20 text-[10px] font-bold text-primary"
					>{i + 1}</span
				>
				<select
					value={step.agent_name}
					onchange={(e) => updateStep(i, 'agent_name', (e.target as HTMLSelectElement).value)}
					class="rounded border border-border bg-background px-1.5 py-0.5 text-[10px] text-foreground"
				>
					{#each agents as agent}
						<option value={agent.name}>{agent.name}</option>
					{/each}
					{#if agents.length === 0}
						<option value={step.agent_name}>{step.agent_name}</option>
					{/if}
				</select>
				<input
					type="text"
					value={step.prompt_template}
					onchange={(e) => updateStep(i, 'prompt_template', (e.target as HTMLInputElement).value)}
					class="flex-1 rounded border border-border bg-background px-1.5 py-0.5 text-[10px] text-foreground"
					placeholder="Prompt template"
				/>
				<button
					onclick={() => removeStep(i)}
					class="text-destructive hover:text-destructive/80 text-xs"
					title="Remove step">x</button
				>
			</div>
		{/each}
	</div>

	<button
		onclick={addStep}
		class="w-full rounded-md border border-dashed border-border py-1.5 text-[10px] text-muted-foreground hover:border-primary/50 hover:text-foreground transition-colors"
	>
		+ Add Step
	</button>
</div>
