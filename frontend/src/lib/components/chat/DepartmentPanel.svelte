<script lang="ts">
	import { onMount } from 'svelte';
	import { getAgents, createAgent, deleteAgent, getSkills, createSkill, deleteSkill, getRules, createRule, updateRule, deleteRule, getDeptEvents, getDeptConfig, updateDeptConfig } from '$lib/api';
	import type { Agent, Skill, Rule, Event, DepartmentConfig } from '$lib/api';

	let {
		dept,
		title,
		icon,
		color,
		quickActions = []
	}: {
		dept: string;
		title: string;
		icon: string;
		color: string; // "emerald" | "purple" | "amber" | "cyan" | "indigo"
		quickActions: { label: string; prompt: string }[];
	} = $props();

	let activeTab = $state('actions');
	let agents: Agent[] = $state([]);
	let skills: Skill[] = $state([]);
	let rules: Rule[] = $state([]);
	let events: Event[] = $state([]);
	let config: DepartmentConfig | null = $state(null);

	// Create forms
	let showCreateAgent = $state(false);
	let newAgentName = $state('');
	let newAgentRole = $state('');
	let newAgentModel = $state('sonnet');
	let newAgentInstructions = $state('');

	let showCreateSkill = $state(false);
	let newSkillName = $state('');
	let newSkillDesc = $state('');
	let newSkillPrompt = $state('');

	let showCreateRule = $state(false);
	let newRuleName = $state('');
	let newRuleContent = $state('');

	onMount(() => {
		loadAgents();
		loadSkills();
		loadRules();
		loadConfig();
	});

	async function loadAgents() { try { agents = await getAgents(dept); } catch { agents = []; } }
	async function loadSkills() { try { skills = await getSkills(dept); } catch { skills = []; } }
	async function loadRules() { try { rules = await getRules(dept); } catch { rules = []; } }
	async function loadEvents() { try { events = await getDeptEvents(dept); } catch { events = []; } }
	async function loadConfig() { try { config = await getDeptConfig(dept); } catch {} }

	function sendQuickAction(prompt: string) {
		document.dispatchEvent(new CustomEvent('dept-quick-action', { detail: { prompt }, bubbles: true }));
	}

	async function handleCreateAgent() {
		if (!newAgentName.trim()) return;
		await createAgent({ name: newAgentName.trim(), role: newAgentRole, model: newAgentModel, instructions: newAgentInstructions, metadata: { engine: dept } });
		newAgentName = ''; newAgentRole = ''; newAgentInstructions = ''; showCreateAgent = false;
		await loadAgents();
	}

	async function handleDeleteAgent(id: string) {
		await deleteAgent(id);
		await loadAgents();
	}

	async function handleCreateSkill() {
		if (!newSkillName.trim()) return;
		await createSkill({ id: '', name: newSkillName.trim(), description: newSkillDesc, prompt_template: newSkillPrompt, metadata: { engine: dept } });
		newSkillName = ''; newSkillDesc = ''; newSkillPrompt = ''; showCreateSkill = false;
		await loadSkills();
	}

	async function handleDeleteSkill(id: string) {
		await deleteSkill(id);
		await loadSkills();
	}

	async function handleCreateRule() {
		if (!newRuleName.trim()) return;
		await createRule({ id: '', name: newRuleName.trim(), content: newRuleContent, enabled: true, metadata: { engine: dept } });
		newRuleName = ''; newRuleContent = ''; showCreateRule = false;
		await loadRules();
	}

	async function handleToggleRule(rule: Rule) {
		await updateRule(rule.id, { ...rule, enabled: !rule.enabled });
		await loadRules();
	}

	async function handleDeleteRule(id: string) {
		await deleteRule(id);
		await loadRules();
	}

	async function addDir() {
		if (!config) return;
		const dir = prompt('Add directory path:');
		if (dir && !config.add_dirs.includes(dir)) {
			config.add_dirs = [...config.add_dirs, dir];
			config = await updateDeptConfig(dept, config);
		}
	}

	async function removeDir(dir: string) {
		if (!config) return;
		config.add_dirs = config.add_dirs.filter(d => d !== dir);
		config = await updateDeptConfig(dept, config);
	}

	function formatTime(iso: string): string {
		try { return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }); }
		catch { return iso; }
	}
</script>

<div class="flex w-72 flex-shrink-0 flex-col border-r border-[var(--r-border-default)] bg-[var(--r-bg-surface)]">
	<!-- Header -->
	<div class="border-b border-[var(--r-border-default)] px-4 py-3">
		<div class="flex items-center gap-2">
			<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-{color}-600/20 text-sm font-bold text-{color}-400">{icon}</div>
			<div>
				<h2 class="text-sm font-semibold text-[var(--r-fg-default)]">{title}</h2>
				<p class="text-[10px] text-[var(--r-fg-subtle)]">{dept} department</p>
			</div>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-[var(--r-border-default)] overflow-x-auto">
		{#each [
			{ id: 'actions', label: 'Actions' },
			{ id: 'agents', label: `Agents (${agents.length})` },
			{ id: 'skills', label: `Skills (${skills.length})` },
			{ id: 'rules', label: `Rules (${rules.length})` },
			{ id: 'projects', label: 'Dirs' },
			{ id: 'events', label: 'Events' },
		] as tab}
			<button
				onclick={() => { activeTab = tab.id; if (tab.id === 'events') loadEvents(); }}
				class="flex-shrink-0 px-2.5 py-2 text-[10px] font-medium transition-colors border-b-2
					{activeTab === tab.id
					? `border-${color}-500 text-${color}-300`
					: 'border-transparent text-[var(--r-fg-muted)] hover:text-[var(--r-fg-default)]'}"
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		<!-- ACTIONS -->
		{#if activeTab === 'actions'}
			<div class="p-3 space-y-1">
				{#each quickActions as action}
					<button onclick={() => sendQuickAction(action.prompt)} class="w-full rounded-lg bg-[var(--r-bg-raised)] px-3 py-2 text-left transition-colors hover:bg-{color}-900/15 group">
						<p class="text-xs font-medium text-[var(--r-fg-default)] group-hover:text-{color}-300">{action.label}</p>
					</button>
				{/each}
			</div>

		<!-- AGENTS -->
		{:else if activeTab === 'agents'}
			<div class="p-3 space-y-2">
				<button onclick={() => showCreateAgent = !showCreateAgent} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-1.5 text-xs text-[var(--r-fg-subtle)] hover:border-{color}-500/30 hover:text-[var(--r-fg-default)]">
					+ New Agent
				</button>
				{#if showCreateAgent}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-3 space-y-2">
						<input bind:value={newAgentName} placeholder="Agent name" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none" />
						<input bind:value={newAgentRole} placeholder="Role description" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none" />
						<select bind:value={newAgentModel} class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)]">
							<option value="sonnet">Sonnet</option>
							<option value="opus">Opus</option>
							<option value="haiku">Haiku</option>
						</select>
						<textarea bind:value={newAgentInstructions} placeholder="System prompt / instructions" rows="3" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none resize-none"></textarea>
						<button onclick={handleCreateAgent} class="w-full rounded-md bg-{color}-600 py-1 text-xs font-medium text-white hover:bg-{color}-500">Create</button>
					</div>
				{/if}
				{#each agents as agent}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-2.5 group">
						<div class="flex items-center justify-between mb-1">
							<span class="text-xs font-medium text-[var(--r-fg-default)]">{agent.name}</span>
							<div class="flex items-center gap-1">
								<span class="rounded bg-{color}-900/30 px-1.5 py-0.5 text-[9px] text-{color}-400">{agent.default_model.model}</span>
								<button onclick={() => handleDeleteAgent(agent.id)} class="hidden group-hover:block text-[var(--r-fg-subtle)] hover:text-danger-400 text-[10px]">x</button>
							</div>
						</div>
						<p class="text-[10px] text-[var(--r-fg-muted)]">{agent.role}</p>
					</div>
				{/each}
				{#if agents.length === 0 && !showCreateAgent}
					<p class="text-center text-[10px] text-[var(--r-fg-subtle)] py-2">No agents. Create one above.</p>
				{/if}
			</div>

		<!-- SKILLS -->
		{:else if activeTab === 'skills'}
			<div class="p-3 space-y-2">
				<button onclick={() => showCreateSkill = !showCreateSkill} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-1.5 text-xs text-[var(--r-fg-subtle)] hover:border-{color}-500/30 hover:text-[var(--r-fg-default)]">
					+ New Skill
				</button>
				{#if showCreateSkill}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-3 space-y-2">
						<input bind:value={newSkillName} placeholder="Skill name (e.g. /wire-engine)" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none" />
						<input bind:value={newSkillDesc} placeholder="Description" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none" />
						<textarea bind:value={newSkillPrompt} placeholder="Prompt template" rows="3" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none resize-none"></textarea>
						<button onclick={handleCreateSkill} class="w-full rounded-md bg-{color}-600 py-1 text-xs font-medium text-white hover:bg-{color}-500">Create</button>
					</div>
				{/if}
				{#each skills as skill}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-2.5 transition-colors hover:bg-{color}-900/15 group cursor-pointer" role="button" tabindex="0" onclick={() => sendQuickAction(skill.prompt_template)} onkeydown={(e) => { if (e.key === 'Enter') sendQuickAction(skill.prompt_template); }}>
						<div class="flex items-center justify-between">
							<span class="text-xs font-mono font-medium text-{color}-400">{skill.name}</span>
							<button onclick={(e) => { e.stopPropagation(); handleDeleteSkill(skill.id); }} class="hidden group-hover:block text-[var(--r-fg-subtle)] hover:text-danger-400 text-[10px]">x</button>
						</div>
						<p class="text-[10px] text-[var(--r-fg-muted)]">{skill.description}</p>
					</div>
				{/each}
				{#if skills.length === 0 && !showCreateSkill}
					<p class="text-center text-[10px] text-[var(--r-fg-subtle)] py-2">No skills. Create one above.</p>
				{/if}
			</div>

		<!-- RULES -->
		{:else if activeTab === 'rules'}
			<div class="p-3 space-y-2">
				<button onclick={() => showCreateRule = !showCreateRule} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-1.5 text-xs text-[var(--r-fg-subtle)] hover:border-{color}-500/30 hover:text-[var(--r-fg-default)]">
					+ New Rule
				</button>
				{#if showCreateRule}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-3 space-y-2">
						<input bind:value={newRuleName} placeholder="Rule name" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none" />
						<textarea bind:value={newRuleContent} placeholder="Rule content (injected into system prompt)" rows="3" class="w-full rounded-md border border-[var(--r-border-default)] bg-[var(--r-bg-base)] px-2 py-1 text-xs text-[var(--r-fg-default)] focus:outline-none resize-none"></textarea>
						<button onclick={handleCreateRule} class="w-full rounded-md bg-{color}-600 py-1 text-xs font-medium text-white hover:bg-{color}-500">Create</button>
					</div>
				{/if}
				{#each rules as rule}
					<div class="rounded-lg bg-[var(--r-bg-raised)] p-2.5 group">
						<div class="flex items-center justify-between mb-1">
							<span class="text-xs font-medium text-[var(--r-fg-default)] {!rule.enabled ? 'line-through opacity-50' : ''}">{rule.name}</span>
							<div class="flex items-center gap-1">
								<button onclick={() => handleToggleRule(rule)} class="rounded px-1.5 py-0.5 text-[9px] {rule.enabled ? `bg-${color}-900/30 text-${color}-400` : 'bg-[var(--r-bg-surface)] text-[var(--r-fg-subtle)]'}">
									{rule.enabled ? 'on' : 'off'}
								</button>
								<button onclick={() => handleDeleteRule(rule.id)} class="hidden group-hover:block text-[var(--r-fg-subtle)] hover:text-danger-400 text-[10px]">x</button>
							</div>
						</div>
						<p class="text-[10px] text-[var(--r-fg-muted)] {!rule.enabled ? 'opacity-50' : ''}">{rule.content.slice(0, 80)}{rule.content.length > 80 ? '...' : ''}</p>
					</div>
				{/each}
				{#if rules.length === 0 && !showCreateRule}
					<p class="text-center text-[10px] text-[var(--r-fg-subtle)] py-2">No rules. Rules get injected into system prompts.</p>
				{/if}
			</div>

		<!-- PROJECTS / DIRS -->
		{:else if activeTab === 'projects'}
			<div class="p-3 space-y-2">
				<p class="text-[10px] text-[var(--r-fg-subtle)]">Working directories (--add-dir).</p>
				{#if config}
					{#each config.add_dirs as dir}
						<div class="flex items-center justify-between rounded-lg bg-[var(--r-bg-raised)] px-3 py-2">
							<span class="text-xs font-mono text-[var(--r-fg-default)]">{dir}</span>
							<button onclick={() => removeDir(dir)} class="text-[var(--r-fg-subtle)] hover:text-danger-400 text-xs">x</button>
						</div>
					{/each}
					<button onclick={addDir} class="w-full rounded-lg border border-dashed border-[var(--r-border-default)] py-2 text-xs text-[var(--r-fg-subtle)] hover:text-[var(--r-fg-default)]">
						+ Add directory
					</button>
				{/if}
			</div>

		<!-- EVENTS -->
		{:else if activeTab === 'events'}
			<div class="p-3 space-y-2">
				{#if events.length === 0}
					<p class="text-xs text-[var(--r-fg-subtle)] text-center py-4">No events yet.</p>
				{:else}
					{#each events as event}
						<div class="rounded-lg bg-[var(--r-bg-raised)] p-2">
							<div class="flex items-center gap-1.5">
								<span class="rounded bg-{color}-900/30 px-1 py-0.5 text-[9px] font-mono text-{color}-400">{event.kind}</span>
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
