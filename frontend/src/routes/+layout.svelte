<script lang="ts">
	import '../app.css';
	import { onMount, type Snippet } from 'svelte';
	import { getSessions, createSession } from '$lib/api';
	import { sessions, activeSession } from '$lib/stores';

	let { children }: { children: Snippet } = $props();

	let showNewSession = $state(false);
	let newName = $state('');
	let newKind = $state('Project');
	let loading = $state(true);
	let error = $state('');

	const navItems = [
		{ href: '/chat', label: 'Chat', icon: '>' },
		{ href: '/', label: 'Dashboard', icon: '~' },
		{ href: '/forge', label: 'Forge', icon: '=' },
		{ href: '/code', label: 'Code', icon: '#' },
		{ href: '/harvest', label: 'Harvest', icon: '$' },
		{ href: '/content', label: 'Content', icon: '*' },
		{ href: '/gtm', label: 'GTM', icon: '^' },
		{ href: '/settings', label: 'Settings', icon: '%' }
	];

	let currentSessions: import('$lib/api').SessionSummary[] = $state([]);
	let currentActive: import('$lib/api').SessionSummary | null = $state(null);

	sessions.subscribe((v) => (currentSessions = v));
	activeSession.subscribe((v) => (currentActive = v));

	onMount(async () => {
		try {
			const list = await getSessions();
			sessions.set(list);
			if (list.length > 0) {
				activeSession.set(list[0]);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load sessions';
		} finally {
			loading = false;
		}
	});

	async function handleCreateSession() {
		if (!newName.trim()) return;
		try {
			await createSession(newName.trim(), newKind);
			const list = await getSessions();
			sessions.set(list);
			if (!currentActive && list.length > 0) {
				activeSession.set(list[0]);
			}
			newName = '';
			showNewSession = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create session';
		}
	}

	function selectSession(event: globalThis.Event) {
		const target = event.target as HTMLSelectElement;
		const id = target.value;
		const found = currentSessions.find((s) => s.id === id);
		if (found) activeSession.set(found);
	}
</script>

<div class="flex h-screen bg-gray-950 text-gray-100">
	<!-- Sidebar -->
	<aside class="flex w-64 flex-shrink-0 flex-col border-r border-gray-800 bg-gray-900">
		<!-- Logo -->
		<div class="flex items-center gap-3 border-b border-gray-800 px-5 py-4">
			<div
				class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-600 text-sm font-bold"
			>
				R
			</div>
			<span class="text-lg font-semibold tracking-tight">RUSVEL</span>
		</div>

		<!-- Session Switcher -->
		<div class="border-b border-gray-800 px-4 py-3">
			<label for="session-select" class="mb-1 block text-xs font-medium text-gray-400">Session</label>
			{#if loading}
				<div class="text-sm text-gray-500">Loading...</div>
			{:else if currentSessions.length === 0}
				<div class="text-sm text-gray-500">No sessions</div>
			{:else}
				<select
					id="session-select"
					onchange={selectSession}
					value={currentActive?.id ?? ''}
					class="w-full rounded-md border border-gray-700 bg-gray-800 px-2 py-1.5 text-sm text-gray-200 focus:border-indigo-500 focus:outline-none"
				>
					{#each currentSessions as session}
						<option value={session.id}>{session.name}</option>
					{/each}
				</select>
			{/if}
			<button
				onclick={() => (showNewSession = !showNewSession)}
				class="mt-2 w-full rounded-md bg-gray-800 px-2 py-1 text-xs text-gray-400 hover:bg-gray-700 hover:text-gray-200"
			>
				+ New Session
			</button>

			{#if showNewSession}
				<div class="mt-2 space-y-2">
					<input
						bind:value={newName}
						placeholder="Session name"
						class="w-full rounded-md border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200 focus:border-indigo-500 focus:outline-none"
					/>
					<select
						bind:value={newKind}
						class="w-full rounded-md border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
					>
						<option>Project</option>
						<option>Lead</option>
						<option>ContentCampaign</option>
						<option>General</option>
					</select>
					<button
						onclick={handleCreateSession}
						class="w-full rounded-md bg-indigo-600 px-2 py-1 text-sm font-medium hover:bg-indigo-500"
					>
						Create
					</button>
				</div>
			{/if}
		</div>

		<!-- Navigation -->
		<nav class="flex-1 overflow-y-auto px-3 py-3">
			{#each navItems as item}
				<a
					href={item.href}
					class="mb-0.5 flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-gray-400 transition-colors hover:bg-gray-800 hover:text-gray-100"
				>
					<span class="w-5 text-center font-mono text-xs text-gray-600">{item.icon}</span>
					{item.label}
				</a>
			{/each}
		</nav>

		<!-- Status -->
		<div class="border-t border-gray-800 px-4 py-3">
			{#if error}
				<div class="text-xs text-red-400">{error}</div>
			{:else}
				<div class="text-xs text-gray-600">API: localhost:3000</div>
			{/if}
		</div>
	</aside>

	<!-- Main Content -->
	<main class="flex-1 overflow-y-auto">
		{@render children()}
	</main>
</div>
