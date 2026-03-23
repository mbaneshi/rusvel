<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commandPaletteOpen, sessions, activeSession } from '$lib/stores';
	import { createSession } from '$lib/api';

	let isOpen = $state(false);
	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement | undefined = $state(undefined);

	commandPaletteOpen.subscribe((v) => {
		isOpen = v;
		if (v) {
			query = '';
			selectedIndex = 0;
			setTimeout(() => inputEl?.focus(), 50);
		}
	});

	interface Command {
		id: string;
		label: string;
		group: string;
		icon: string;
		action: () => void;
	}

	const commands: Command[] = [
		// Navigation
		{
			id: 'nav-dashboard',
			label: 'Dashboard',
			group: 'Navigation',
			icon: '~',
			action: () => navigate('/')
		},
		{
			id: 'nav-chat',
			label: 'Chat (God Agent)',
			group: 'Navigation',
			icon: '>',
			action: () => navigate('/chat')
		},
		{
			id: 'nav-forge',
			label: 'Forge Department',
			group: 'Navigation',
			icon: '=',
			action: () => navigate('/forge')
		},
		{
			id: 'nav-code',
			label: 'Code Department',
			group: 'Navigation',
			icon: '#',
			action: () => navigate('/code')
		},
		{
			id: 'nav-harvest',
			label: 'Harvest Department',
			group: 'Navigation',
			icon: '$',
			action: () => navigate('/harvest')
		},
		{
			id: 'nav-content',
			label: 'Content Department',
			group: 'Navigation',
			icon: '*',
			action: () => navigate('/content')
		},
		{
			id: 'nav-gtm',
			label: 'GTM Department',
			group: 'Navigation',
			icon: '^',
			action: () => navigate('/gtm')
		},
		{
			id: 'nav-finance',
			label: 'Finance Department',
			group: 'Navigation',
			icon: '%',
			action: () => navigate('/finance')
		},
		{
			id: 'nav-product',
			label: 'Product Department',
			group: 'Navigation',
			icon: '@',
			action: () => navigate('/product')
		},
		{
			id: 'nav-growth',
			label: 'Growth Department',
			group: 'Navigation',
			icon: '&',
			action: () => navigate('/growth')
		},
		{
			id: 'nav-distro',
			label: 'Distribution Department',
			group: 'Navigation',
			icon: '!',
			action: () => navigate('/distro')
		},
		{
			id: 'nav-legal',
			label: 'Legal Department',
			group: 'Navigation',
			icon: '\u00A7',
			action: () => navigate('/legal')
		},
		{
			id: 'nav-support',
			label: 'Support Department',
			group: 'Navigation',
			icon: '?',
			action: () => navigate('/support')
		},
		{
			id: 'nav-infra',
			label: 'Infra Department',
			group: 'Navigation',
			icon: '>',
			action: () => navigate('/infra')
		},
		{
			id: 'nav-settings',
			label: 'Settings',
			group: 'Navigation',
			icon: '%',
			action: () => navigate('/settings')
		},
		// Actions
		{
			id: 'act-new-session',
			label: 'Create New Session',
			group: 'Actions',
			icon: '+',
			action: () => handleCreateSession()
		},
		{
			id: 'act-plan',
			label: 'Generate Daily Plan',
			group: 'Actions',
			icon: '=',
			action: () => navigate('/forge')
		},
		{
			id: 'act-new-chat',
			label: 'New Chat',
			group: 'Actions',
			icon: '>',
			action: () => navigate('/chat')
		}
	];

	let filtered = $derived(
		query.trim() === ''
			? commands
			: commands.filter(
					(c) =>
						c.label.toLowerCase().includes(query.toLowerCase()) ||
						c.group.toLowerCase().includes(query.toLowerCase())
				)
	);

	let groups = $derived(
		[...new Set(filtered.map((c) => c.group))].map((g) => ({
			name: g,
			items: filtered.filter((c) => c.group === g)
		}))
	);

	function navigate(path: string) {
		close();
		goto(path);
	}

	function close() {
		commandPaletteOpen.set(false);
	}

	async function handleCreateSession() {
		close();
		const name = `session-${Date.now()}`;
		await createSession(name, 'General');
		goto('/');
	}

	function runSelected() {
		if (filtered[selectedIndex]) {
			filtered[selectedIndex].action();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			runSelected();
		} else if (e.key === 'Escape') {
			close();
		}
	}

	onMount(() => {
		function globalKeydown(e: KeyboardEvent) {
			if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
				e.preventDefault();
				commandPaletteOpen.update((v) => !v);
			}
		}
		window.addEventListener('keydown', globalKeydown);
		return () => window.removeEventListener('keydown', globalKeydown);
	});
</script>

{#if isOpen}
	<!-- Backdrop -->
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[20vh]"
		onclick={close}
		onkeydown={(e) => e.key === 'Escape' && close()}
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-label="Command palette"
	>
		<!-- Panel -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="w-full max-w-lg rounded-xl border border-[var(--border)] bg-[var(--card)] shadow-2xl"
			onclick={(e) => e.stopPropagation()}
		>
			<!-- Search input -->
			<div class="flex items-center gap-3 border-b border-[var(--border)] px-4 py-3">
				<svg
					class="h-4 w-4 text-[var(--muted-foreground)]"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"><circle cx="6.5" cy="6.5" r="4" /><path d="M10 10l3.5 3.5" /></svg
				>
				<input
					bind:this={inputEl}
					bind:value={query}
					onkeydown={handleKeydown}
					placeholder="Search commands..."
					class="flex-1 bg-transparent text-sm text-[var(--foreground)] placeholder-[var(--muted-foreground)] focus:outline-none"
				/>
				<kbd
					class="rounded border border-[var(--border)] bg-[var(--secondary)] px-1.5 py-0.5 text-[10px] text-[var(--muted-foreground)]"
					>esc</kbd
				>
			</div>

			<!-- Results -->
			<div class="max-h-72 overflow-y-auto py-2">
				{#if filtered.length === 0}
					<p class="px-4 py-6 text-center text-sm text-[var(--muted-foreground)]">
						No results found
					</p>
				{:else}
					{#each groups as group}
						<div class="px-3 pb-1 pt-2">
							<p
								class="px-2 text-[10px] font-medium uppercase tracking-wider text-[var(--muted-foreground)]"
							>
								{group.name}
							</p>
						</div>
						{#each group.items as item, i}
							{@const globalIdx = filtered.indexOf(item)}
							<button
								onclick={item.action}
								onmouseenter={() => (selectedIndex = globalIdx)}
								class="flex w-full items-center gap-3 px-5 py-2 text-left text-sm transition-colors
									{globalIdx === selectedIndex
									? 'bg-[var(--secondary)] text-[var(--foreground)]'
									: 'text-[var(--muted-foreground)] hover:bg-[var(--secondary)]'}"
							>
								<span
									class="flex h-5 w-5 items-center justify-center rounded bg-[var(--secondary)] font-mono text-[10px] text-[var(--muted-foreground)]"
									>{item.icon}</span
								>
								{item.label}
							</button>
						{/each}
					{/each}
				{/if}
			</div>

			<!-- Footer -->
			<div
				class="flex items-center justify-between border-t border-[var(--border)] px-4 py-2 text-[10px] text-[var(--muted-foreground)]"
			>
				<div class="flex items-center gap-2">
					<span><kbd class="rounded border border-[var(--border)] px-1">↑↓</kbd> navigate</span>
					<span><kbd class="rounded border border-[var(--border)] px-1">↵</kbd> select</span>
				</div>
				<span><kbd class="rounded border border-[var(--border)] px-1">⌘K</kbd> toggle</span>
			</div>
		</div>
	</div>
{/if}
