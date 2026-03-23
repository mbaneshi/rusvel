<script lang="ts">
	import { activeSession } from '$lib/stores';
	import { getDeptEvents } from '$lib/api';
	import type { Event } from '$lib/api';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	let events: Event[] = $state([]);
	let activeTab = $state('chat');

	activeSession.subscribe((v) => {
		currentSession = v;
		if (v) loadEvents();
	});

	async function loadEvents() {
		try {
			events = await getDeptEvents('code');
		} catch {
			events = [];
		}
	}

	function formatTime(iso: string): string {
		try { return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }); }
		catch { return iso; }
	}
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session from the sidebar to begin.</p>
		</div>
	{:else}
		<!-- Left panel: Department info & events -->
		<div class="flex w-64 flex-shrink-0 flex-col border-r border-[var(--r-border-default)] bg-[var(--r-bg-surface)]">
			<!-- Department header -->
			<div class="border-b border-[var(--r-border-default)] p-4">
				<div class="flex items-center gap-2 mb-2">
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-600/20 text-sm font-bold text-emerald-400">#</div>
					<div>
						<h2 class="text-sm font-semibold text-[var(--r-fg-default)]">Code Department</h2>
						<p class="text-[10px] text-[var(--r-fg-subtle)]">Claude Code with full tools</p>
					</div>
				</div>
			</div>

			<!-- Tabs -->
			<div class="flex border-b border-[var(--r-border-default)]">
				{#each [{ id: 'chat', label: 'Chat' }, { id: 'events', label: 'Events' }] as tab}
					<button
						onclick={() => activeTab = tab.id}
						class="flex-1 py-2 text-xs font-medium transition-colors border-b-2
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
				{#if activeTab === 'events'}
					<div class="p-3 space-y-2">
						{#if events.length === 0}
							<p class="text-xs text-[var(--r-fg-subtle)] text-center py-4">No code events yet.</p>
						{:else}
							{#each events as event}
								<div class="rounded-lg bg-[var(--r-bg-raised)] p-2">
									<div class="flex items-center gap-1.5">
										<span class="rounded bg-emerald-900/30 px-1 py-0.5 text-[9px] font-mono text-emerald-400">{event.source}</span>
										<span class="text-[9px] text-[var(--r-fg-subtle)]">{formatTime(event.created_at)}</span>
									</div>
									<p class="mt-0.5 text-[10px] text-[var(--r-fg-muted)]">{event.kind}</p>
								</div>
							{/each}
						{/if}
						<button onclick={loadEvents} class="w-full rounded-md bg-[var(--r-bg-raised)] py-1.5 text-[10px] text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)]">
							Refresh
						</button>
					</div>
				{:else}
					<div class="p-3 space-y-3">
						<div class="rounded-lg bg-[var(--r-bg-raised)] p-3">
							<h4 class="text-xs font-medium text-[var(--r-fg-default)] mb-1">Capabilities</h4>
							<ul class="space-y-1 text-[10px] text-[var(--r-fg-muted)]">
								<li>Read, Write, Edit files</li>
								<li>Run shell commands</li>
								<li>Search codebases (grep/glob)</li>
								<li>Web search & fetch</li>
								<li>Spawn sub-agents</li>
								<li>Background tasks</li>
							</ul>
						</div>
						<div class="rounded-lg bg-[var(--r-bg-raised)] p-3">
							<h4 class="text-xs font-medium text-[var(--r-fg-default)] mb-1">Quick Actions</h4>
							<div class="space-y-1">
								{#each ['Analyze codebase', 'Run tests', 'Review architecture', 'Fix a bug'] as action}
									<button class="w-full rounded-md bg-[var(--r-bg-surface)] px-2 py-1.5 text-left text-[10px] text-[var(--r-fg-muted)] hover:text-[var(--r-fg-default)] hover:bg-emerald-900/10 transition-colors">
										{action}
									</button>
								{/each}
							</div>
						</div>
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
