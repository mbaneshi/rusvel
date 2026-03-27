<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		MessageSquare,
		LayoutDashboard,
		Database,
		GitBranch,
		Settings,
		ClipboardCheck,
		TerminalSquare
	} from 'lucide-svelte';
	import { get } from 'svelte/store';
	import { departments, pendingApprovalCount, commandPaletteOpen } from '$lib/stores';
	import { deptHref } from '$lib/api';
	import type { DepartmentDef } from '$lib/api';
	import DeptIcon from '$lib/components/DeptIcon.svelte';

	let deptList: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (deptList = v));

	const globalLinks = [
		{ href: '/', label: 'Dashboard', icon: LayoutDashboard, tour: 'nav-dashboard' },
		{ href: '/chat', label: 'Chat', icon: MessageSquare, tour: 'nav-chat' },
		{ href: '/approvals', label: 'Approvals', icon: ClipboardCheck, tour: '' },
		{ href: '/database/schema', label: 'Database', icon: Database, tour: '' },
		{ href: '/flows', label: 'Flows', icon: GitBranch, tour: '' },
		{ href: '/terminal', label: 'Terminal', icon: TerminalSquare, tour: '' }
	] as const;

	function linkActive(href: string): boolean {
		const p = page.url.pathname;
		if (href === '/') return p === '/';
		return p === href || p.startsWith(`${href}/`);
	}

	function deptActive(id: string): boolean {
		const p = page.url.pathname;
		return p === `/dept/${id}` || p.startsWith(`/dept/${id}/`);
	}

	onMount(() => {
		const onKey = (e: KeyboardEvent) => {
			if (!e.altKey || e.metaKey || e.ctrlKey || e.shiftKey) return;
			if (get(commandPaletteOpen)) return;
			const el = e.target as HTMLElement | null;
			if (el?.closest('input, textarea, select, [contenteditable="true"]')) return;
			const k = e.key;
			if (k < '1' || k > '9') return;
			const idx = Number.parseInt(k, 10) - 1;
			const d = deptList[idx];
			if (!d) return;
			e.preventDefault();
			goto(deptHref(d.id));
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});
</script>

<aside
	class="flex w-12 shrink-0 flex-col items-center gap-0.5 border-r border-border bg-sidebar py-2"
	aria-label="Primary"
>
	{#each globalLinks as item}
		{@const Icon = item.icon}
		{@const active = linkActive(item.href)}
		<a
			href={item.href}
			data-tour={item.tour || undefined}
			title={item.label}
			class="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-md transition-colors
				{active
				? 'bg-sidebar-primary/20 text-sidebar-primary'
				: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
		>
			<Icon size={18} strokeWidth={1.75} />
			{#if item.href === '/approvals' && $pendingApprovalCount > 0}
				<span
					class="absolute right-0.5 top-0.5 flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-destructive px-0.5 text-[8px] font-bold leading-none text-destructive-foreground"
				>
					{$pendingApprovalCount > 99 ? '99+' : $pendingApprovalCount}
				</span>
			{/if}
		</a>
	{/each}

	<div class="my-1 h-px w-7 shrink-0 bg-border" aria-hidden="true"></div>

	<div class="flex min-h-0 w-full flex-1 flex-col items-center gap-0.5 overflow-y-auto overflow-x-hidden px-0.5">
		{#each deptList as d, i}
			<a
				href={deptHref(d.id)}
				data-tour={d.id === 'forge' ? 'nav-forge' : undefined}
				title={i < 9 ? `${d.name} (Alt+${i + 1})` : d.name}
				class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md transition-colors
					{deptActive(d.id)
					? 'bg-sidebar-primary/20 text-sidebar-primary'
					: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
			>
				<DeptIcon deptId={d.id} size={18} strokeWidth={1.75} />
			</a>
		{/each}
	</div>

	<a
		href="/settings"
		title="Settings"
		class="mt-auto flex h-9 w-9 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
			{linkActive('/settings') ? 'bg-sidebar-primary/20 text-sidebar-primary' : ''}"
	>
		<Settings size={18} strokeWidth={1.75} />
	</a>
</aside>
