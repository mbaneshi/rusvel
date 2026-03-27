<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { departments, contextPanelOpen } from '$lib/stores';
	import type { DepartmentDef } from '$lib/api';
	import { MessageSquare, Sliders, Activity, LayoutGrid, Calendar, PanelRightOpen } from 'lucide-svelte';
	import { deptExtraSections } from '$lib/departmentManifest';
	import ContextPanel from '$lib/components/shell/ContextPanel.svelte';

	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let { children }: { children: Snippet } = $props();

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
	let base = $derived(`/dept/${page.params.id}`);

	let extras = $derived(deptExtraSections[page.params.id ?? ''] ?? []);

	function sectionActive(segment: string): boolean {
		const p = page.url.pathname;
		if (segment === 'chat') return p.endsWith('/chat') || p === `/dept/${page.params.id}`;
		return p.endsWith(`/${segment}`);
	}

	onMount(() => {
		if (!browser) return;
		const onKey = (e: KeyboardEvent) => {
			if (!(e.metaKey || e.ctrlKey) || e.key.toLowerCase() !== 'j') return;
			const el = e.target as HTMLElement | null;
			if (el?.closest('input, textarea, [contenteditable="true"]')) return;
			e.preventDefault();
			contextPanelOpen.update((v) => !v);
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});
</script>

{#if !dept}
	<div class="flex h-full items-center justify-center">
		<p class="text-sm text-[var(--muted-foreground)]">Department not found.</p>
	</div>
{:else}
	<div class="flex h-full min-h-0">
		<aside
			class="flex w-48 shrink-0 flex-col gap-1 border-r border-border bg-sidebar/40 px-2 py-3"
			aria-label="Department sections"
		>
			<p class="mb-1 px-2 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
				{dept.name}
			</p>
			<a
				href="{base}/chat"
				class="flex items-center gap-2 rounded-md px-2 py-2 text-sm transition-colors
					{sectionActive('chat')
					? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
					: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
			>
				<MessageSquare size={16} strokeWidth={1.75} class="shrink-0" />
				Chat
			</a>
			<a
				href="{base}/config"
				class="flex items-center gap-2 rounded-md px-2 py-2 text-sm transition-colors
					{sectionActive('config')
					? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
					: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
			>
				<Sliders size={16} strokeWidth={1.75} class="shrink-0" />
				Config
			</a>
			<a
				href="{base}/events"
				class="flex items-center gap-2 rounded-md px-2 py-2 text-sm transition-colors
					{sectionActive('events')
					? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
					: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
			>
				<Activity size={16} strokeWidth={1.75} class="shrink-0" />
				Events
			</a>
			{#each extras as ex}
				<a
					href="{base}/{ex.segment}"
					class="flex items-center gap-2 rounded-md px-2 py-2 text-sm transition-colors
						{sectionActive(ex.segment)
						? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
						: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
				>
					{#if ex.segment === 'pipeline'}
						<LayoutGrid size={16} strokeWidth={1.75} class="shrink-0" />
					{:else if ex.segment === 'calendar'}
						<Calendar size={16} strokeWidth={1.75} class="shrink-0" />
					{/if}
					{ex.label}
				</a>
			{/each}

			<button
				type="button"
				class="mt-auto flex items-center gap-2 rounded-md px-2 py-2 text-left text-xs text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-foreground"
				onclick={() => contextPanelOpen.update((v) => !v)}
				title="Toggle context panel (⌘J / Ctrl+J)"
			>
				<PanelRightOpen size={16} strokeWidth={1.75} class="shrink-0" />
				<span>Context</span>
				<kbd class="ml-auto hidden rounded border border-border bg-secondary/80 px-1 font-mono text-[9px] lg:inline"
					>⌘J</kbd
				>
			</button>
		</aside>

		<div class="flex min-h-0 min-w-0 flex-1">
			<div class="min-h-0 min-w-0 flex-1 overflow-hidden">
				{@render children()}
			</div>
			{#if $contextPanelOpen}
				<ContextPanel deptId={dept.id} deptTitle={dept.title} />
			{:else}
				<button
					type="button"
					class="flex w-9 shrink-0 flex-col items-center justify-center gap-1 border-l border-border bg-muted/20 py-2 text-muted-foreground hover:bg-muted/40"
					onclick={() => contextPanelOpen.set(true)}
					title="Open context panel (⌘J)"
				>
					<PanelRightOpen size={18} strokeWidth={1.75} />
				</button>
			{/if}
		</div>
	</div>
{/if}
