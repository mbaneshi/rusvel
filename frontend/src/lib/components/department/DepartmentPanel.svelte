<script lang="ts">
	import { onMount } from 'svelte';
	import { panelOpen, panelWidth } from '$lib/stores';
	import { getDeptConfig } from '$lib/api';
	import type { DepartmentConfig } from '$lib/api';
	import DeptHelpTooltip from '$lib/components/onboarding/DeptHelpTooltip.svelte';
	import { getDeptColor } from './colors';
	import ActionsTab from './ActionsTab.svelte';
	import AgentsTab from './AgentsTab.svelte';
	import SkillsTab from './SkillsTab.svelte';
	import RulesTab from './RulesTab.svelte';
	import McpTab from './McpTab.svelte';
	import HooksTab from './HooksTab.svelte';
	import WorkflowsTab from './WorkflowsTab.svelte';
	import DirsTab from './DirsTab.svelte';
	import EventsTab from './EventsTab.svelte';

	let {
		dept,
		title,
		icon,
		color,
		quickActions = [],
		tabs = ['actions', 'agents', 'workflows', 'skills', 'rules', 'mcp', 'hooks', 'dirs', 'events'],
		helpDescription = '',
		helpPrompts = [],
	}: {
		dept: string;
		title: string;
		icon: string;
		color: string;
		quickActions: { label: string; prompt: string }[];
		tabs?: string[];
		helpDescription?: string;
		helpPrompts?: string[];
	} = $props();

	let activeTab = $state('actions');
	let config: DepartmentConfig | null = $state(null);
	let isOpen = $state(true);
	let width = $state(288);
	let resizing = $state(false);

	const deptHsl = $derived(getDeptColor(color));

	panelOpen.subscribe((v) => (isOpen = v));
	panelWidth.subscribe((v) => (width = v));

	onMount(async () => {
		try { config = await getDeptConfig(dept); } catch {}
	});

	function togglePanel() {
		panelOpen.update((v) => !v);
	}

	function startResize(e: MouseEvent) {
		e.preventDefault();
		resizing = true;
		const startX = e.clientX;
		const startWidth = width;
		const onMove = (ev: MouseEvent) => {
			const delta = ev.clientX - startX;
			const newWidth = Math.max(200, Math.min(500, startWidth + delta));
			panelWidth.set(newWidth);
		};
		const onUp = () => {
			resizing = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	const tabDefs = $derived([
		{ id: 'actions', label: 'Actions' },
		{ id: 'agents', label: 'Agents' },
		{ id: 'workflows', label: 'Flows' },
		{ id: 'skills', label: 'Skills' },
		{ id: 'rules', label: 'Rules' },
		{ id: 'mcp', label: 'MCP' },
		{ id: 'hooks', label: 'Hooks' },
		{ id: 'projects', label: 'Dirs' },
		{ id: 'events', label: 'Events' },
	].filter(t => tabs.includes(t.id) || (t.id === 'projects' && tabs.includes('dirs'))));
</script>

{#if !isOpen}
	<div class="flex w-10 flex-shrink-0 flex-col items-center border-r border-border bg-card py-3 gap-2">
		<button
			onclick={togglePanel}
			class="flex h-8 w-8 items-center justify-center rounded-lg text-sm font-bold"
			style="background: hsl({deptHsl} / 0.2); color: hsl({deptHsl})"
			title="Expand {title}"
		>
			{icon}
		</button>
	</div>
{:else}
<div
	class="flex flex-shrink-0 flex-col border-r border-border bg-card relative"
	class:select-none={resizing}
	style="width: {width}px; --dept: {deptHsl}"
>
	<!-- Header -->
	<div class="border-b border-border px-4 py-3">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2 min-w-0">
				<div
					class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg text-sm font-bold"
					style="background: hsl({deptHsl} / 0.2); color: hsl({deptHsl})"
				>{icon}</div>
				<div class="min-w-0">
					<h2 class="text-sm font-semibold text-foreground truncate">{title}</h2>
					<p class="text-[10px] text-muted-foreground">{dept} department</p>
				</div>
			</div>
			<div class="flex items-center gap-1">
				{#if helpDescription}
					<DeptHelpTooltip dept={dept} description={helpDescription} prompts={helpPrompts} />
				{/if}
				<button
					onclick={togglePanel}
					class="rounded-md p-1 text-muted-foreground hover:bg-secondary hover:text-foreground"
					title="Collapse panel"
				>
					<svg class="h-4 w-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 3L5 8l5 5" /></svg>
				</button>
			</div>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-border overflow-x-auto">
		{#each tabDefs as tab}
			<button
				onclick={() => activeTab = tab.id}
				class="flex-shrink-0 px-2.5 py-2 text-[10px] font-medium transition-colors border-b-2
					{activeTab === tab.id
					? 'text-foreground'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
				style={activeTab === tab.id ? `border-color: hsl(${deptHsl}); color: hsl(${deptHsl})` : ''}
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		{#if activeTab === 'actions'}
			<ActionsTab {dept} {quickActions} {deptHsl} />
		{:else if activeTab === 'agents'}
			<AgentsTab {dept} {deptHsl} />
		{:else if activeTab === 'skills'}
			<SkillsTab {dept} {deptHsl} />
		{:else if activeTab === 'rules'}
			<RulesTab {dept} {deptHsl} />
		{:else if activeTab === 'mcp'}
			<McpTab {dept} />
		{:else if activeTab === 'hooks'}
			<HooksTab {dept} />
		{:else if activeTab === 'workflows'}
			<WorkflowsTab {dept} {deptHsl} />
		{:else if activeTab === 'projects'}
			<DirsTab {dept} {config} />
		{:else if activeTab === 'events'}
			<EventsTab {dept} {deptHsl} />
		{/if}
	</div>

	<!-- Resize handle -->
	<div
		onmousedown={startResize}
		role="button"
		tabindex="0"
		class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 active:bg-primary/70 transition-colors"
	></div>
</div>
{/if}
