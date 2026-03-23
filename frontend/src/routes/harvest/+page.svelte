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
		{ name: 'gig-scanner', model: 'sonnet', desc: 'Scan freelance platforms for matching opportunities.' },
		{ name: 'proposal-writer', model: 'opus', desc: 'Draft compelling proposals tailored to each gig.' },
		{ name: 'opportunity-scorer', model: 'haiku', desc: 'Score and rank opportunities by fit and revenue potential.' },
	];

	// Pre-built skills from strategy doc 05
	const prebuiltSkills = [
		{ name: '/scan-gigs', desc: 'Scan platforms for new opportunities' },
		{ name: '/score-opportunity', desc: 'Score and rank an opportunity' },
		{ name: '/draft-proposal', desc: 'Draft a proposal for a specific gig' },
		{ name: '/pipeline-report', desc: 'Generate pipeline status report' },
	];

	// Quick actions that send as chat messages
	const quickActions = [
		{ label: 'Scan Upwork for Rust gigs', prompt: 'Scan Upwork for Rust-related gigs. Filter for remote, hourly or fixed-price, and rank by fit.' },
		{ label: 'Score this opportunity', prompt: 'Score an opportunity I will describe. Evaluate fit, revenue potential, effort, and strategic value. Ask me for details.' },
		{ label: 'Draft proposal for...', prompt: 'Draft a proposal for a freelance gig. Ask me for the job description and any specific requirements.' },
		{ label: 'Pipeline status', prompt: 'Show the current opportunity pipeline. List all tracked opportunities with status, score, and next action.' },
		{ label: 'Competitor analysis', prompt: 'Analyze competitors in my niche. Look at their positioning, pricing, and recent wins.' },
		{ label: 'Weekly opportunity digest', prompt: 'Generate a weekly digest of new opportunities, pipeline changes, and recommended actions.' },
	];

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) {
			loadEvents();
			loadConfig();
		}
	});

	async function loadEvents() {
		try { events = await getDeptEvents('harvest'); } catch { events = []; }
	}

	async function loadConfig() {
		try { config = await getDeptConfig('harvest'); } catch { /* defaults */ }
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
			config = await updateDeptConfig('harvest', config);
		}
	}

	async function removeDir(dir: string) {
		if (!config) return;
		config.add_dirs = config.add_dirs.filter(d => d !== dir);
		config = await updateDeptConfig('harvest', config);
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
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-600/20 text-sm font-bold text-amber-400">$</div>
					<div>
						<h2 class="text-sm font-semibold text-[var(--r-fg-default)]">Harvest Department</h2>
						<p class="text-[10px] text-[var(--r-fg-subtle)]">Discover and capture opportunities</p>
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
							? 'border-amber-500 text-amber-300'
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
								class="w-full rounded-lg bg-[var(--r-bg-raised)] px-3 py-2 text-left transition-colors hover:bg-amber-900/15 group"
							>
								<p class="text-xs font-medium text-[var(--r-fg-default)] group-hover:text-amber-300">{action.label}</p>
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
									<span class="rounded bg-amber-900/30 px-1.5 py-0.5 text-[9px] text-amber-400">{agent.model}</span>
								</div>
								<p class="text-[10px] text-[var(--r-fg-muted)]">{agent.desc}</p>
							</div>
						{/each}
					</div>

				{:else if activeTab === 'skills'}
					<div class="p-3 space-y-2">
						<p class="text-[10px] text-[var(--r-fg-subtle)] mb-2">Custom skills for opportunity discovery workflows.</p>
						{#each prebuiltSkills as skill}
							<button
								onclick={() => sendQuickAction(`Run skill: ${skill.name}. ${skill.desc}`)}
								class="w-full rounded-lg bg-[var(--r-bg-raised)] p-2.5 text-left transition-colors hover:bg-amber-900/15 group"
							>
								<span class="text-xs font-mono font-medium text-amber-400">{skill.name}</span>
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
							<button onclick={addDir} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-2 text-xs text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)] hover:border-amber-500/30">
								+ Add directory
							</button>
						{:else}
							<p class="text-xs text-[var(--r-fg-subtle)]">Loading...</p>
						{/if}
					</div>

				{:else if activeTab === 'events'}
					<div class="p-3 space-y-2">
						{#if events.length === 0}
							<p class="text-xs text-[var(--r-fg-subtle)] text-center py-4">No events yet. Chat with the Harvest department to generate events.</p>
						{:else}
							{#each events as event}
								<div class="rounded-lg bg-[var(--r-bg-raised)] p-2">
									<div class="flex items-center gap-1.5">
										<span class="rounded bg-amber-900/30 px-1 py-0.5 text-[9px] font-mono text-amber-400">{event.kind}</span>
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
			<DepartmentChat dept="harvest" title="Harvest Department" icon="$" />
		</div>
	{/if}
</div>
