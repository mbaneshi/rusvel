<script lang="ts">
	import { activeSession } from '$lib/stores';
	import { getDeptEvents, getDeptConfig, updateDeptConfig } from '$lib/api';
	import type { Event, DepartmentConfig } from '$lib/api';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	let events: Event[] = $state([]);
	let config: DepartmentConfig | null = $state(null);
	let activeTab = $state('actions');
	let chatRef: DepartmentChat | undefined = $state(undefined);

	// Pre-built agents from strategy doc 04
	const prebuiltAgents = [
		{ name: 'mission-planner', model: 'opus', desc: 'Generate daily plans and align tasks with strategic goals.' },
		{ name: 'persona-manager', model: 'sonnet', desc: 'Hire and configure agent personas for specific tasks.' },
		{ name: 'system-monitor', model: 'haiku', desc: 'Monitor system health, resource usage, and performance.' },
	];

	// Pre-built skills from strategy doc 05
	const prebuiltSkills = [
		{ name: '/daily-plan', desc: 'Generate today\'s mission plan' },
		{ name: '/goal-review', desc: 'Review progress on active goals' },
		{ name: '/hire-persona', desc: 'Hire an agent persona for a task' },
		{ name: '/weekly-review', desc: 'Run weekly review and retrospective' },
	];

	// Quick actions that send as chat messages
	const quickActions = [
		{ label: 'Generate daily plan', prompt: 'Generate a daily plan for today. Prioritize tasks by impact, check active goals, and suggest focus areas.' },
		{ label: 'Review goals progress', prompt: 'Review progress on all active goals. Show completion percentage, blockers, and recommended next steps.' },
		{ label: 'Hire persona for task', prompt: 'I need to hire an agent persona for a specific task. Ask me what the task is and suggest the best persona configuration.' },
		{ label: 'System health check', prompt: 'Run a system health check. Report on database size, active sessions, job queue status, and any issues.' },
		{ label: 'Weekly review', prompt: 'Run the weekly review. Summarize what was accomplished, what was missed, lessons learned, and plan for next week.' },
		{ label: 'What should I focus on?', prompt: 'Based on active goals, pending tasks, and current priorities, what should I focus on right now? Give me the single most impactful thing.' },
	];

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) {
			loadEvents();
			loadConfig();
		}
	});

	async function loadEvents() {
		try { events = await getDeptEvents('forge'); } catch { events = []; }
	}

	async function loadConfig() {
		try { config = await getDeptConfig('forge'); } catch { /* defaults */ }
	}

	function sendQuickAction(prompt: string) {
		const event = new CustomEvent('dept-quick-action', { detail: { prompt }, bubbles: true });
		document.dispatchEvent(event);
	}

	function formatTime(iso: string): string {
		try { return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }); }
		catch { return iso; }
	}

	async function addDir() {
		if (!config) return;
		const dir = prompt('Add project directory (relative or absolute path):');
		if (dir && !config.add_dirs.includes(dir)) {
			config.add_dirs = [...config.add_dirs, dir];
			config = await updateDeptConfig('forge', config);
		}
	}

	async function removeDir(dir: string) {
		if (!config) return;
		config.add_dirs = config.add_dirs.filter(d => d !== dir);
		config = await updateDeptConfig('forge', config);
	}
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<!-- Left panel -->
		<div class="flex w-72 flex-shrink-0 flex-col border-r border-[var(--r-border-default)] bg-[var(--r-bg-surface)]">
			<!-- Header -->
			<div class="border-b border-[var(--r-border-default)] px-4 py-3">
				<div class="flex items-center gap-2">
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-600/20 text-sm font-bold text-indigo-400">=</div>
					<div>
						<h2 class="text-sm font-semibold text-[var(--r-fg-default)]">Forge Department</h2>
						<p class="text-[10px] text-[var(--r-fg-subtle)]">Planning, goals, and orchestration</p>
					</div>
				</div>
			</div>

			<!-- Tabs -->
			<div class="flex border-b border-[var(--r-border-default)] overflow-x-auto">
				{#each [
					{ id: 'actions', label: 'Actions' },
					{ id: 'agents', label: 'Agents' },
					{ id: 'skills', label: 'Skills' },
					{ id: 'projects', label: 'Projects' },
					{ id: 'events', label: 'Events' },
				] as tab}
					<button
						onclick={() => { activeTab = tab.id; if (tab.id === 'events') loadEvents(); }}
						class="flex-shrink-0 px-3 py-2 text-[10px] font-medium transition-colors border-b-2
							{activeTab === tab.id
							? 'border-indigo-500 text-indigo-300'
							: 'border-transparent text-[var(--r-fg-muted)] hover:text-[var(--r-fg-default)]'}"
					>
						{tab.label}
					</button>
				{/each}
			</div>

			<!-- Tab content -->
			<div class="flex-1 overflow-y-auto">
				{#if activeTab === 'actions'}
					<div class="p-3 space-y-1">
						{#each quickActions as action}
							<button
								onclick={() => sendQuickAction(action.prompt)}
								class="w-full rounded-lg bg-[var(--r-bg-raised)] px-3 py-2 text-left transition-colors hover:bg-indigo-900/15 group"
							>
								<p class="text-xs font-medium text-[var(--r-fg-default)] group-hover:text-indigo-300">{action.label}</p>
							</button>
						{/each}
					</div>

				{:else if activeTab === 'agents'}
					<div class="p-3 space-y-2">
						<p class="text-[10px] text-[var(--r-fg-subtle)] mb-2">Pre-built agents from strategy reports. These can be dispatched as sub-agents.</p>
						{#each prebuiltAgents as agent}
							<div class="rounded-lg bg-[var(--r-bg-raised)] p-2.5">
								<div class="flex items-center justify-between mb-1">
									<span class="text-xs font-medium text-[var(--r-fg-default)]">{agent.name}</span>
									<span class="rounded bg-indigo-900/30 px-1.5 py-0.5 text-[9px] text-indigo-400">{agent.model}</span>
								</div>
								<p class="text-[10px] text-[var(--r-fg-muted)]">{agent.desc}</p>
							</div>
						{/each}
					</div>

				{:else if activeTab === 'skills'}
					<div class="p-3 space-y-2">
						<p class="text-[10px] text-[var(--r-fg-subtle)] mb-2">Custom skills for planning and orchestration workflows.</p>
						{#each prebuiltSkills as skill}
							<button
								onclick={() => sendQuickAction(`Run skill: ${skill.name}. ${skill.desc}`)}
								class="w-full rounded-lg bg-[var(--r-bg-raised)] p-2.5 text-left transition-colors hover:bg-indigo-900/15 group"
							>
								<span class="text-xs font-mono font-medium text-indigo-400">{skill.name}</span>
								<p class="text-[10px] text-[var(--r-fg-muted)] group-hover:text-[var(--r-fg-default)]">{skill.desc}</p>
							</button>
						{/each}
					</div>

				{:else if activeTab === 'projects'}
					<div class="p-3 space-y-2">
						<p class="text-[10px] text-[var(--r-fg-subtle)] mb-2">Working directories passed as --add-dir to Claude.</p>
						{#if config}
							{#each config.add_dirs as dir}
								<div class="flex items-center justify-between rounded-lg bg-[var(--r-bg-raised)] px-3 py-2">
									<span class="text-xs font-mono text-[var(--r-fg-default)]">{dir}</span>
									<button onclick={() => removeDir(dir)} class="text-[var(--r-fg-subtle)] hover:text-danger-400 text-xs">x</button>
								</div>
							{/each}
							<button onclick={addDir} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-2 text-xs text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)] hover:border-indigo-500/30">
								+ Add directory
							</button>
						{:else}
							<p class="text-xs text-[var(--r-fg-subtle)]">Loading...</p>
						{/if}
					</div>

				{:else if activeTab === 'events'}
					<div class="p-3 space-y-2">
						{#if events.length === 0}
							<p class="text-xs text-[var(--r-fg-subtle)] text-center py-4">No events yet. Chat with the Forge department to generate events.</p>
						{:else}
							{#each events as event}
								<div class="rounded-lg bg-[var(--r-bg-raised)] p-2">
									<div class="flex items-center gap-1.5">
										<span class="rounded bg-indigo-900/30 px-1 py-0.5 text-[9px] font-mono text-indigo-400">{event.kind}</span>
										<span class="text-[9px] text-[var(--r-fg-subtle)]">{formatTime(event.created_at)}</span>
									</div>
								</div>
							{/each}
						{/if}
						<button onclick={loadEvents} class="w-full rounded-md bg-[var(--r-bg-raised)] py-1.5 text-[10px] text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)]">Refresh</button>
					</div>
				{/if}
			</div>
		</div>

		<!-- Main: Department Chat -->
		<div class="flex-1">
			<DepartmentChat dept="forge" title="Forge Department" icon="=" />
		</div>
	{/if}
</div>
