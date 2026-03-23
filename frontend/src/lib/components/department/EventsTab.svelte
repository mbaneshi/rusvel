<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getDeptEvents } from '$lib/api';
	import type { Event } from '$lib/api';

	let { dept, deptHsl }: { dept: string; deptHsl: string } = $props();

	let events: Event[] = $state([]);

	onMount(() => {
		loadEvents();
	});

	async function loadEvents() {
		try {
			events = await getDeptEvents(dept);
		} catch (e) {
			events = [];
			toast.error(`Failed to load events: ${e instanceof Error ? e.message : e}`);
		}
	}

	function formatTime(iso: string): string {
		try {
			return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		} catch {
			return iso;
		}
	}
</script>

<div class="p-3 space-y-2">
	{#if events.length === 0}
		<p class="text-xs text-muted-foreground text-center py-4">No events yet.</p>
	{:else}
		{#each events as event}
			<div class="rounded-lg bg-secondary p-2">
				<div class="flex items-center gap-1.5">
					<span
						class="rounded px-1 py-0.5 text-[9px] font-mono"
						style="background-color: hsl({deptHsl}/.15); color: hsl({deptHsl})">{event.kind}</span
					>
					<span class="text-[9px] text-muted-foreground">{formatTime(event.created_at)}</span>
				</div>
			</div>
		{/each}
	{/if}
	<button
		onclick={loadEvents}
		class="w-full rounded-md bg-secondary py-1.5 text-[10px] text-muted-foreground hover:text-foreground"
		>Refresh</button
	>
</div>
