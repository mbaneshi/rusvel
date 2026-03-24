<script lang="ts">
	import '../app.css';
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { getSessions, createSession, getDepartments } from '$lib/api';
	import type { DepartmentDef } from '$lib/api';
	import { sessions, activeSession, sidebarOpen, sidebarWidth, departments } from '$lib/stores';
	import OnboardingChecklist from '$lib/components/onboarding/OnboardingChecklist.svelte';
	import ProductTour from '$lib/components/onboarding/ProductTour.svelte';
	import CommandPalette from '$lib/components/onboarding/CommandPalette.svelte';
	import { Toaster } from 'svelte-sonner';
	import { PaneGroup, Pane, PaneResizer } from 'paneforge';

	let { children }: { children: Snippet } = $props();

	let showNewSession = $state(false);
	let newName = $state('');
	let newKind = $state('Project');
	let loading = $state(true);
	let error = $state('');
	let isOpen = $state(true);
	let width = $state(256);

	// Static nav items (non-department pages)
	const staticNavBefore = [
		{ href: '/chat', label: 'Chat', icon: '>', tour: 'nav-chat' },
		{ href: '/', label: 'Dashboard', icon: '~', tour: 'nav-dashboard' },
		{ href: '/database/schema', label: 'Database', icon: 'B', tour: '' }
	];
	const staticNavAfter = [
		{ href: '/settings', label: 'Settings', icon: '%', tour: 'nav-settings' }
	];

	// Department nav items generated from registry
	let deptList: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (deptList = v));

	let navItems = $derived([
		...staticNavBefore,
		...deptList.map((d) => ({
			href: `/dept/${d.id}`,
			label: d.name,
			icon: d.icon,
			tour: d.id === 'forge' ? 'nav-forge' : ''
		})),
		...staticNavAfter
	]);

	let currentSessions: import('$lib/api').SessionSummary[] = $state([]);
	let currentActive: import('$lib/api').SessionSummary | null = $state(null);

	sessions.subscribe((v) => (currentSessions = v));
	activeSession.subscribe((v) => (currentActive = v));
	sidebarOpen.subscribe((v) => (isOpen = v));
	sidebarWidth.subscribe((v) => (width = v));

	onMount(async () => {
		try {
			const [list, depts] = await Promise.all([getSessions(), getDepartments()]);
			sessions.set(list);
			departments.set(depts);
			if (list.length > 0) {
				activeSession.set(list[0]);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
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

	function toggleSidebar() {
		sidebarOpen.update((v) => !v);
	}

	// Convert pixel width to percentage of viewport for PaneForge
	function pxToPercent(px: number): number {
		if (typeof window === 'undefined') return 18;
		return (px / window.innerWidth) * 100;
	}

	function percentToPx(pct: number): number {
		if (typeof window === 'undefined') return 256;
		return (pct / 100) * window.innerWidth;
	}

	// Sidebar size in percentage (PaneForge works in %)
	let sidebarDefaultSize = $state(18); // ~256px on 1440px screen

	onMount(() => {
		sidebarDefaultSize = pxToPercent(width);
	});

	function handleSidebarResize(size: number) {
		const px = percentToPx(size);
		width = px;
		sidebarWidth.set(px);
	}
</script>

<div class="h-screen bg-background text-foreground flex">
	{#if isOpen}
		<PaneGroup direction="horizontal" class="h-full w-full">
			<Pane
				defaultSize={sidebarDefaultSize}
				minSize={3}
				maxSize={28}
				onResize={handleSidebarResize}
				class="flex flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
			>
				<!-- Logo -->
				<div
					class="flex items-center justify-between border-b border-sidebar-border px-4 py-3"
					data-tour="sidebar-logo"
				>
					<div class="flex items-center gap-3">
						<div
							class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-sm font-bold text-primary-foreground"
						>
							R
						</div>
						{#if width > 100}
							<span class="text-lg font-semibold tracking-tight">RUSVEL</span>
						{/if}
					</div>
					<button
						onclick={toggleSidebar}
						class="rounded-md p-1 text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
						title="Collapse sidebar"
					>
						<svg
							class="h-4 w-4"
							viewBox="0 0 16 16"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"><path d="M10 3L5 8l5 5" /></svg
						>
					</button>
				</div>

				<!-- Session Switcher -->
				{#if width > 100}
					<div class="border-b border-sidebar-border px-4 py-3" data-tour="session-switcher">
						<label for="session-select" class="mb-1 block text-xs font-medium text-muted-foreground"
							>Session</label
						>
						{#if loading}
							<div class="text-sm text-muted-foreground">Loading...</div>
						{:else if currentSessions.length === 0}
							<div class="text-sm text-muted-foreground">No sessions</div>
						{:else}
							<select
								id="session-select"
								onchange={selectSession}
								value={currentActive?.id ?? ''}
								class="w-full rounded-md border border-border bg-secondary px-2 py-1.5 text-sm text-foreground focus:border-primary focus:outline-none"
							>
								{#each currentSessions as session}
									<option value={session.id}>{session.name}</option>
								{/each}
							</select>
						{/if}
						<button
							onclick={() => (showNewSession = !showNewSession)}
							class="mt-2 w-full rounded-md bg-secondary px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground"
						>
							+ New Session
						</button>

						{#if showNewSession}
							<div class="mt-2 space-y-2">
								<input
									bind:value={newName}
									placeholder="Session name"
									class="w-full rounded-md border border-border bg-secondary px-2 py-1 text-sm text-foreground focus:border-primary focus:outline-none"
								/>
								<select
									bind:value={newKind}
									class="w-full rounded-md border border-border bg-secondary px-2 py-1 text-sm text-foreground"
								>
									<option>Project</option>
									<option>Lead</option>
									<option>ContentCampaign</option>
									<option>General</option>
								</select>
								<button
									onclick={handleCreateSession}
									class="w-full rounded-md bg-primary px-2 py-1 text-sm font-medium text-primary-foreground hover:bg-primary/90"
								>
									Create
								</button>
							</div>
						{/if}
					</div>
				{/if}

				<!-- Navigation -->
				<nav class="flex-1 overflow-y-auto px-2 py-2">
					{#each navItems as item}
						{@const isActive =
							item.href === '/'
								? page.url.pathname === '/'
								: page.url.pathname.startsWith(item.href)}
						<a
							href={item.href}
							data-tour={item.tour || undefined}
							class="mb-0.5 flex items-center gap-3 rounded-lg px-3 py-1.5 text-sm transition-colors
								{isActive
								? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
								: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
							title={item.label}
						>
							<span
								class="w-5 flex-shrink-0 text-center font-mono text-xs {isActive
									? 'text-sidebar-primary'
									: 'text-muted-foreground/50'}">{item.icon}</span
							>
							{#if width > 100}
								{item.label}
							{/if}
						</a>
					{/each}
				</nav>

				<!-- Status + Cmd+K hint -->
				{#if width > 100}
					<div class="border-t border-sidebar-border px-4 py-3">
						{#if error}
							<div class="text-xs text-destructive">{error}</div>
						{:else}
							<div class="flex items-center justify-between">
								<span class="text-xs text-muted-foreground/50">API: localhost:3000</span>
								<kbd
									class="rounded border border-border bg-secondary px-1.5 py-0.5 text-[10px] text-muted-foreground"
									>⌘K</kbd
								>
							</div>
						{/if}
					</div>
				{/if}
			</Pane>

			<PaneResizer
				class="w-1 cursor-col-resize bg-transparent hover:bg-primary/50 active:bg-primary/70 transition-colors"
			/>

			<Pane class="overflow-hidden">
				<main class="h-full overflow-hidden">
					{@render children()}
				</main>
			</Pane>
		</PaneGroup>
	{:else}
		<!-- Collapsed sidebar — just icons -->
		<aside
			class="flex w-12 flex-shrink-0 flex-col items-center border-r border-sidebar-border bg-sidebar py-3 gap-1"
		>
			<button
				onclick={toggleSidebar}
				class="mb-2 flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-sm font-bold text-primary-foreground hover:bg-primary/90"
				title="Expand sidebar"
				data-tour="sidebar-logo"
			>
				R
			</button>
			{#each navItems as item}
				{@const isActive =
					item.href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(item.href)}
				<a
					href={item.href}
					data-tour={item.tour || undefined}
					class="flex h-8 w-8 items-center justify-center rounded-lg text-xs transition-colors
						{isActive
						? 'bg-sidebar-primary/15 text-sidebar-primary'
						: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
					title={item.label}
				>
					<span class="font-mono">{item.icon}</span>
				</a>
			{/each}
		</aside>

		<!-- Main Content when sidebar collapsed -->
		<main class="flex-1 overflow-hidden">
			{@render children()}
		</main>
	{/if}
</div>

<!-- Overlays -->
<Toaster richColors position="bottom-right" theme="dark" />
<CommandPalette />
<OnboardingChecklist />
<ProductTour />
