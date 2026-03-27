<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import { getContentScheduled, type ScheduledPostRow } from '$lib/api';
	import { toast } from 'svelte-sonner';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let posts: ScheduledPostRow[] = $state([]);
	let loading = $state(true);
	let view: 'week' | 'all' = $state('week');

	let deptId = $derived(page.params.id);
	let isContent = $derived(deptId === 'content');

	function platformLabel(p: ScheduledPostRow['platform']): string {
		if (typeof p === 'string') return p;
		if (p && typeof p === 'object') {
			if ('LinkedIn' in p) return 'LinkedIn';
			if ('Twitter' in p) return 'Twitter';
			if ('DevTo' in p) return 'DEV.to';
			return JSON.stringify(p);
		}
		return '—';
	}

	function platformClass(p: ScheduledPostRow['platform']): string {
		const label = platformLabel(p).toLowerCase();
		if (label.includes('linkedin')) return 'border-sky-500/40 bg-sky-500/10 text-sky-200';
		if (label.includes('twitter')) return 'border-cyan-500/40 bg-cyan-500/10 text-cyan-200';
		if (label.includes('dev')) return 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200';
		return 'border-border bg-secondary/60 text-muted-foreground';
	}

	function groupByDay(list: ScheduledPostRow[]): Map<string, ScheduledPostRow[]> {
		const m = new Map<string, ScheduledPostRow[]>();
		for (const x of list) {
			try {
				const d = new Date(x.publish_at);
				const key = d.toLocaleDateString(undefined, {
					weekday: 'short',
					year: 'numeric',
					month: 'short',
					day: 'numeric'
				});
				if (!m.has(key)) m.set(key, []);
				m.get(key)!.push(x);
			} catch {
				const key = 'Unknown date';
				if (!m.has(key)) m.set(key, []);
				m.get(key)!.push(x);
			}
		}
		return m;
	}

	let filtered = $derived.by(() => {
		if (view !== 'week' || posts.length === 0) return posts;
		const now = new Date();
		const start = new Date(now);
		start.setDate(now.getDate() - now.getDay());
		start.setHours(0, 0, 0, 0);
		const end = new Date(start);
		end.setDate(start.getDate() + 7);
		return posts.filter((p) => {
			const t = new Date(p.publish_at).getTime();
			return t >= start.getTime() && t < end.getTime();
		});
	});

	let grouped = $derived(groupByDay([...filtered].sort((a, b) => +new Date(a.publish_at) - +new Date(b.publish_at))));

	async function load() {
		if (!sessionId || !isContent) return;
		loading = true;
		try {
			posts = await getContentScheduled(sessionId);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load calendar');
			posts = [];
		} finally {
			loading = false;
		}
	}

	onMount(() => void load());

	$effect(() => {
		if (sessionId && isContent) void load();
	});
</script>

<div class="flex h-full min-h-0 flex-col overflow-auto">
	<div class="flex flex-wrap items-center justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">Content calendar</h1>
			<p class="text-xs text-muted-foreground">Scheduled posts from the content engine.</p>
		</div>
		<div class="flex gap-1 rounded-md border border-border p-0.5 text-xs">
			<button
				type="button"
				class="rounded px-2 py-1 {view === 'week' ? 'bg-secondary text-foreground' : 'text-muted-foreground'}"
				onclick={() => (view = 'week')}
			>
				This week
			</button>
			<button
				type="button"
				class="rounded px-2 py-1 {view === 'all' ? 'bg-secondary text-foreground' : 'text-muted-foreground'}"
				onclick={() => (view = 'all')}
			>
				All
			</button>
		</div>
	</div>

	{#if !isContent}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">
				Calendar lives under <span class="font-mono text-foreground">/dept/content/calendar</span>.
			</p>
		</div>
	{:else if !sessionId}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">Select a session in the top bar.</p>
		</div>
	{:else if loading}
		<div class="p-6 text-sm text-muted-foreground">Loading…</div>
	{:else if filtered.length === 0}
		<div class="p-6 text-sm text-muted-foreground">No scheduled posts. Schedule from the content engine or API.</div>
	{:else}
		<div class="space-y-6 p-4">
			{#each [...grouped.entries()] as [day, items]}
				<div>
					<h2 class="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">{day}</h2>
					<ul class="space-y-2">
						{#each items as p}
							<li
								class="flex flex-wrap items-center justify-between gap-2 rounded-lg border px-3 py-2 {platformClass(
									p.platform
								)}"
							>
								<span class="font-mono text-[11px]">{p.content_id.slice(0, 8)}…</span>
								<span class="text-[11px]">{platformLabel(p.platform)}</span>
								<span class="text-[11px] opacity-80">
									{new Date(p.publish_at).toLocaleString()}
								</span>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	{/if}
</div>
