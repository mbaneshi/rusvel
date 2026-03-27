<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { departments } from '$lib/stores';
	import { deptHref } from '$lib/api';
	import type { DepartmentDef } from '$lib/api';
	import DeptIcon from '$lib/components/DeptIcon.svelte';

	let deptList: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (deptList = v));

	function isDeptActive(id: string): boolean {
		const p = page.url.pathname;
		return p === `/dept/${id}` || p.startsWith(`/dept/${id}/`);
	}

	onMount(() => {
		const onKey = (e: KeyboardEvent) => {
			if (!e.altKey || e.metaKey || e.ctrlKey) return;
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

<nav
	class="flex h-11 shrink-0 items-center gap-1 overflow-x-auto border-b border-border bg-sidebar/80 px-2 py-1"
	aria-label="Departments"
>
	{#each deptList as d, i}
		<a
			href={deptHref(d.id)}
			data-tour={d.id === 'forge' ? 'nav-forge' : undefined}
			class="flex shrink-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-xs transition-colors
				{isDeptActive(d.id)
				? 'bg-sidebar-primary/15 text-sidebar-primary font-medium'
				: 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
			title="{d.name} (Alt+{i < 9 ? i + 1 : ''})"
		>
			<span class="flex h-5 w-5 items-center justify-center">
				<DeptIcon deptId={d.id} size={16} strokeWidth={1.75} />
			</span>
			<span class="max-w-[7rem] truncate sm:max-w-[10rem]">{d.name}</span>
			{#if i < 9}
				<kbd
					class="hidden rounded border border-border bg-secondary/80 px-1 font-mono text-[9px] text-muted-foreground xl:inline"
					>⌥{i + 1}</kbd
				>
			{/if}
		</a>
	{/each}
</nav>
