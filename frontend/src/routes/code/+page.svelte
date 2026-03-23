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
		{ name: 'rust-engine', model: 'opus', desc: 'Build Rust engine crates. Enforces hexagonal architecture.' },
		{ name: 'svelte-ui', model: 'sonnet', desc: 'Build SvelteKit 5 pages using RUSVEL design system.' },
		{ name: 'test-writer', model: 'sonnet', desc: 'Write comprehensive tests for crates and frontend.' },
		{ name: 'security-auditor', model: 'opus', desc: 'Review code for vulnerabilities (OWASP, injection, auth).' },
		{ name: 'api-builder', model: 'sonnet', desc: 'Build Axum API endpoints following RUSVEL patterns.' },
	];

	// Pre-built skills from strategy doc 05
	const prebuiltSkills = [
		{ name: '/wire-engine', desc: 'Scaffold CLI + API + MCP for an engine' },
		{ name: '/new-component', desc: 'Create design system component' },
		{ name: '/new-engine', desc: 'Scaffold a new engine crate' },
		{ name: '/daily-review', desc: 'End-of-day review: done, issues, tomorrow' },
		{ name: '/deploy-check', desc: 'Pre-deploy verification checklist' },
	];

	// Quick actions that send as chat messages
	const quickActions = [
		{ label: 'Analyze this codebase', prompt: 'Analyze the RUSVEL codebase. Show me: total crates, lines of code, test count, and any issues found.' },
		{ label: 'Run all tests', prompt: 'Run `cargo test` and `npm run check` in the frontend. Report results.' },
		{ label: 'Review architecture', prompt: 'Review the current RUSVEL architecture. Check if engines follow hexagonal rules (no adapter imports). List any violations.' },
		{ label: 'Find TODOs', prompt: 'Search the codebase for TODO, FIXME, HACK comments. List them grouped by crate.' },
		{ label: 'Git status', prompt: 'Run `git log --oneline -10` and `git status`. Show what changed recently.' },
		{ label: 'What needs work?', prompt: 'Read docs/status/gap-analysis.md and docs/status/current-state.md. What are the highest priority items to work on next?' },
	];

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) {
			loadEvents();
			loadConfig();
		}
	});

	async function loadEvents() {
		try { events = await getDeptEvents('code'); } catch { events = []; }
	}

	async function loadConfig() {
		try { config = await getDeptConfig('code'); } catch { /* defaults */ }
	}

	function sendQuickAction(prompt: string) {
		// Navigate focus to chat input and programmatically send
		// We use a custom event to communicate with the DepartmentChat
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
			config = await updateDeptConfig('code', config);
		}
	}

	async function removeDir(dir: string) {
		if (!config) return;
		config.add_dirs = config.add_dirs.filter(d => d !== dir);
		config = await updateDeptConfig('code', config);
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
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-600/20 text-sm font-bold text-emerald-400">#</div>
					<div>
						<h2 class="text-sm font-semibold text-[var(--r-fg-default)]">Code Department</h2>
						<p class="text-[10px] text-[var(--r-fg-subtle)]">Full Claude Code capabilities</p>
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
							? 'border-emerald-500 text-emerald-300'
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
								class="w-full rounded-lg bg-[var(--r-bg-raised)] px-3 py-2 text-left transition-colors hover:bg-emerald-900/15 group"
							>
								<p class="text-xs font-medium text-[var(--r-fg-default)] group-hover:text-emerald-300">{action.label}</p>
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
									<span class="rounded bg-emerald-900/30 px-1.5 py-0.5 text-[9px] text-emerald-400">{agent.model}</span>
								</div>
								<p class="text-[10px] text-[var(--r-fg-muted)]">{agent.desc}</p>
							</div>
						{/each}
					</div>

				{:else if activeTab === 'skills'}
					<div class="p-3 space-y-2">
						<p class="text-[10px] text-[var(--r-fg-subtle)] mb-2">Custom skills for RUSVEL development workflows.</p>
						{#each prebuiltSkills as skill}
							<button
								onclick={() => sendQuickAction(`Run skill: ${skill.name}. ${skill.desc}`)}
								class="w-full rounded-lg bg-[var(--r-bg-raised)] p-2.5 text-left transition-colors hover:bg-emerald-900/15 group"
							>
								<span class="text-xs font-mono font-medium text-emerald-400">{skill.name}</span>
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
							<button onclick={addDir} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-2 text-xs text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)] hover:border-emerald-500/30">
								+ Add directory
							</button>
						{:else}
							<p class="text-xs text-[var(--r-fg-subtle)]">Loading...</p>
						{/if}
					</div>

				{:else if activeTab === 'events'}
					<div class="p-3 space-y-2">
						{#if events.length === 0}
							<p class="text-xs text-[var(--r-fg-subtle)] text-center py-4">No events yet. Chat with the Code department to generate events.</p>
						{:else}
							{#each events as event}
								<div class="rounded-lg bg-[var(--r-bg-raised)] p-2">
									<div class="flex items-center gap-1.5">
										<span class="rounded bg-emerald-900/30 px-1 py-0.5 text-[9px] font-mono text-emerald-400">{event.kind}</span>
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
			<DepartmentChat dept="code" title="Code Department" icon="#" />
		</div>
	{/if}
</div>
