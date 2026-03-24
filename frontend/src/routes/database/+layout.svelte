<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';

	let { children }: { children: Snippet } = $props();

	const links = [
		{ href: '/database/schema', label: 'Schema' },
		{ href: '/database/tables', label: 'Tables' },
		{ href: '/database/sql', label: 'SQL' }
	];
</script>

<div class="flex h-full flex-col overflow-hidden bg-background">
	<div class="border-b border-border px-4 py-3">
		<h1 class="text-sm font-semibold text-foreground">Database</h1>
		<p class="text-[10px] text-muted-foreground">RusvelBase — SQLite introspection (local)</p>
		<nav class="mt-2 flex flex-wrap gap-1">
			{#each links as L}
				<a
					href={L.href}
					class="rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors
					{page.url.pathname === L.href
						? 'bg-secondary text-foreground'
						: 'text-muted-foreground hover:bg-secondary/60 hover:text-foreground'}"
				>
					{L.label}
				</a>
			{/each}
		</nav>
	</div>
	<div class="min-h-0 flex-1 overflow-auto">
		{@render children()}
	</div>
</div>
