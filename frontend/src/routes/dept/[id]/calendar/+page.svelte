<script lang="ts">
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import {
		getContentList,
		getContentScheduled,
		type ContentItemRow,
		type ScheduledPostRow
	} from '$lib/api';
	import { toast } from 'svelte-sonner';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let posts: ScheduledPostRow[] = $state([]);
	let contentItems: ContentItemRow[] = $state([]);
	let listSessionId = $state<string | null>(null);
	let loading = $state(true);
	let viewMode: 'week' | 'month' = $state('week');
	let focusDate = $state(new Date());

	let selectedPost = $state<ScheduledPostRow | null>(null);
	let detailItem = $state<ContentItemRow | null>(null);
	let detailLoading = $state(false);

	let deptId = $derived(page.params.id);
	let isContent = $derived(deptId === 'content');

	function weekRangeLocal(d: Date): { from: string; to: string } {
		const local = new Date(d);
		const day = (local.getDay() + 6) % 7;
		const start = new Date(local);
		start.setDate(local.getDate() - day);
		start.setHours(0, 0, 0, 0);
		const end = new Date(start);
		end.setDate(start.getDate() + 6);
		end.setHours(23, 59, 59, 999);
		return { from: start.toISOString(), to: end.toISOString() };
	}

	function monthRangeLocal(d: Date): { from: string; to: string } {
		const y = d.getFullYear();
		const m = d.getMonth();
		const start = new Date(y, m, 1, 0, 0, 0, 0);
		const end = new Date(y, m + 1, 0, 23, 59, 59, 999);
		return { from: start.toISOString(), to: end.toISOString() };
	}

	function weekDaysFrom(focus: Date): Date[] {
		const local = new Date(focus);
		const day = (local.getDay() + 6) % 7;
		const start = new Date(local);
		start.setDate(local.getDate() - day);
		start.setHours(0, 0, 0, 0);
		const days: Date[] = [];
		for (let i = 0; i < 7; i++) {
			const x = new Date(start);
			x.setDate(start.getDate() + i);
			days.push(x);
		}
		return days;
	}

	function monthGridCells(focus: Date): { date: Date; inMonth: boolean }[] {
		const y = focus.getFullYear();
		const m = focus.getMonth();
		const first = new Date(y, m, 1);
		const dow = (first.getDay() + 6) % 7;
		const gridStart = new Date(first);
		gridStart.setDate(1 - dow);
		const cells: { date: Date; inMonth: boolean }[] = [];
		for (let i = 0; i < 42; i++) {
			const d = new Date(gridStart);
			d.setDate(gridStart.getDate() + i);
			cells.push({ date: d, inMonth: d.getMonth() === m });
		}
		return cells;
	}

	function sameLocalDay(a: Date, b: Date): boolean {
		return (
			a.getFullYear() === b.getFullYear() &&
			a.getMonth() === b.getMonth() &&
			a.getDate() === b.getDate()
		);
	}

	function postsOnDay(day: Date): ScheduledPostRow[] {
		return posts.filter((p) => sameLocalDay(new Date(p.publish_at), day));
	}

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
		if (label.includes('linkedin')) return 'border-sky-500/50 bg-sky-500/15 text-sky-100';
		if (label.includes('twitter')) return 'border-cyan-500/50 bg-cyan-500/15 text-cyan-100';
		if (label.includes('dev')) return 'border-emerald-500/50 bg-emerald-500/15 text-emerald-100';
		return 'border-border bg-secondary/60 text-muted-foreground';
	}

	let weekDays = $derived(weekDaysFrom(focusDate));
	let monthCells = $derived(monthGridCells(focusDate));

	let rangeLabel = $derived.by(() => {
		if (viewMode === 'month') {
			return focusDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
		}
		const days = weekDaysFrom(focusDate);
		const a = days[0]!;
		const b = days[6]!;
		const sameYear = a.getFullYear() === b.getFullYear();
		const opts: Intl.DateTimeFormatOptions = sameYear
			? { month: 'short', day: 'numeric' }
			: { month: 'short', day: 'numeric', year: 'numeric' };
		return `${a.toLocaleDateString(undefined, opts)} – ${b.toLocaleDateString(undefined, { ...opts, year: 'numeric' })}`;
	});

	function shiftWeek(delta: number) {
		const d = new Date(focusDate);
		d.setDate(d.getDate() + 7 * delta);
		focusDate = d;
	}

	function shiftMonth(delta: number) {
		const d = new Date(focusDate);
		d.setMonth(d.getMonth() + delta);
		focusDate = d;
	}

	function goToday() {
		focusDate = new Date();
	}

	async function load() {
		if (!sessionId || !isContent) return;
		loading = true;
		try {
			const range =
				viewMode === 'week' ? weekRangeLocal(focusDate) : monthRangeLocal(focusDate);
			if (listSessionId !== sessionId) {
				contentItems = await getContentList(sessionId);
				listSessionId = sessionId;
			}
			posts = await getContentScheduled(sessionId, range.from, range.to);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load calendar');
			posts = [];
		} finally {
			loading = false;
		}
	}

	function closeDetail() {
		selectedPost = null;
		detailItem = null;
	}

	async function openDetail(p: ScheduledPostRow) {
		selectedPost = p;
		detailLoading = true;
		detailItem = null;
		try {
			if (listSessionId !== sessionId || contentItems.length === 0) {
				if (sessionId) {
					contentItems = await getContentList(sessionId);
					listSessionId = sessionId;
				}
			}
			detailItem = contentItems.find((x) => x.id === p.content_id) ?? null;
		} catch {
			toast.error('Could not load draft');
		} finally {
			detailLoading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') closeDetail();
	}

	$effect(() => {
		if (sessionId && isContent) {
			focusDate.getTime();
			viewMode;
			void load();
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-full min-h-0 flex-col overflow-auto">
	<div class="flex flex-wrap items-center justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">Content calendar</h1>
			<p class="text-xs text-muted-foreground">Scheduled posts — week and month views.</p>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<div class="flex gap-1 rounded-md border border-border p-0.5 text-xs">
				<button
					type="button"
					class="rounded px-2 py-1 {viewMode === 'week'
						? 'bg-secondary text-foreground'
						: 'text-muted-foreground'}"
					onclick={() => (viewMode = 'week')}
				>
					Week
				</button>
				<button
					type="button"
					class="rounded px-2 py-1 {viewMode === 'month'
						? 'bg-secondary text-foreground'
						: 'text-muted-foreground'}"
					onclick={() => (viewMode = 'month')}
				>
					Month
				</button>
			</div>
			<div class="flex flex-wrap items-center gap-1 text-xs">
				<button
					type="button"
					class="rounded border border-border px-2 py-1 hover:bg-secondary"
					onclick={() => (viewMode === 'week' ? shiftWeek(-1) : shiftMonth(-1))}
				>
					‹ Prev
				</button>
				<button
					type="button"
					class="rounded border border-border px-2 py-1 hover:bg-secondary"
					onclick={goToday}
				>
					Today
				</button>
				<button
					type="button"
					class="rounded border border-border px-2 py-1 hover:bg-secondary"
					onclick={() => (viewMode === 'week' ? shiftWeek(1) : shiftMonth(1))}
				>
					Next ›
				</button>
			</div>
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
	{:else}
		<div class="border-b border-border px-4 py-2">
			<p class="text-sm font-medium text-foreground">{rangeLabel}</p>
		</div>

		{#if viewMode === 'week'}
			<div class="grid min-h-[280px] grid-cols-7 gap-px border-b border-border bg-border p-4">
				{#each weekDays as day}
					<div class="flex min-h-[220px] min-w-0 flex-col bg-background">
						<div class="border-b border-border px-1 pb-1 text-center">
							<div class="text-[10px] font-medium uppercase text-muted-foreground">
								{day.toLocaleDateString(undefined, { weekday: 'short' })}
							</div>
							<div
								class="text-sm font-semibold {sameLocalDay(day, new Date())
									? 'text-primary'
									: ''}"
							>
								{day.getDate()}
							</div>
						</div>
						<ul class="flex flex-1 flex-col gap-1 overflow-auto p-1">
							{#each postsOnDay(day) as p}
								<li>
									<button
										type="button"
										class="w-full rounded border px-1.5 py-1 text-left text-[11px] leading-snug transition hover:opacity-90 {platformClass(
											p.platform
										)}"
										onclick={() => void openDetail(p)}
									>
										<span class="block truncate font-medium">{platformLabel(p.platform)}</span>
										<span class="block font-mono opacity-80">{p.content_id.slice(0, 8)}…</span>
										<span class="block text-[10px] opacity-75">
											{new Date(p.publish_at).toLocaleTimeString(undefined, {
												hour: 'numeric',
												minute: '2-digit'
											})}
										</span>
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/each}
			</div>
		{:else}
			<div class="grid grid-cols-7 gap-px border-b border-border bg-border p-4 text-[10px]">
				{#each ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as h}
					<div class="bg-background px-1 py-1 text-center font-semibold uppercase text-muted-foreground">
						{h}
					</div>
				{/each}
				{#each monthCells as { date, inMonth }}
					<div
						class="min-h-[88px] bg-background p-1 {inMonth ? '' : 'opacity-40'}"
					>
						<div class="mb-1 text-right text-xs font-medium {sameLocalDay(date, new Date()) ? 'text-primary' : ''}">
							{date.getDate()}
						</div>
						<ul class="space-y-0.5">
							{#each postsOnDay(date) as p}
								<li>
									<button
										type="button"
										class="block w-full truncate rounded border px-0.5 py-0.5 text-left text-[10px] leading-tight {platformClass(
											p.platform
										)}"
										onclick={() => void openDetail(p)}
									>
										{platformLabel(p.platform)}
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/each}
			</div>
		{/if}

		{#if posts.length === 0}
			<div class="p-6 text-sm text-muted-foreground">
				No scheduled posts in this range. Schedule from the content engine or API.
			</div>
		{/if}
	{/if}
</div>

{#if selectedPost}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-40 flex items-end justify-center bg-black/50 sm:items-center sm:p-4"
		onclick={(e) => e.target === e.currentTarget && closeDetail()}
		role="presentation"
	>
		<div
			class="max-h-[85vh] w-full max-w-lg overflow-auto rounded-t-lg border border-border bg-background shadow-xl sm:rounded-lg"
			role="dialog"
			aria-modal="true"
			aria-labelledby="cal-detail-title"
		>
			<div class="flex items-start justify-between gap-2 border-b border-border px-4 py-3">
				<div>
					<h2 id="cal-detail-title" class="text-base font-semibold">Scheduled post</h2>
					<p class="text-xs text-muted-foreground">
						{platformLabel(selectedPost.platform)} · {new Date(selectedPost.publish_at).toLocaleString()}
					</p>
				</div>
				<button
					type="button"
					class="rounded px-2 py-1 text-sm text-muted-foreground hover:bg-secondary hover:text-foreground"
					onclick={closeDetail}
				>
					Close
				</button>
			</div>
			<div class="space-y-3 px-4 py-3 text-sm">
				<div class="flex flex-wrap gap-2 text-xs">
					<span
						class="rounded border px-2 py-0.5 {platformClass(selectedPost.platform)}"
					>
						{platformLabel(selectedPost.platform)}
					</span>
					<span class="rounded border border-border bg-secondary/50 px-2 py-0.5 text-muted-foreground">
						{selectedPost.status}
					</span>
					<span class="font-mono text-[11px] text-muted-foreground">{selectedPost.content_id}</span>
				</div>
				{#if detailLoading}
					<p class="text-muted-foreground">Loading draft…</p>
				{:else if detailItem}
					<div>
						<h3 class="mb-1 font-medium">{detailItem.title || '(Untitled)'}</h3>
						<pre
							class="max-h-[40vh] overflow-auto whitespace-pre-wrap rounded-md border border-border bg-muted/30 p-3 text-xs leading-relaxed"
						>{detailItem.body_markdown || '—'}</pre>
						<p class="mt-2 text-xs text-muted-foreground">
							{detailItem.kind} · {detailItem.status}
							{#if detailItem.scheduled_at}
								· scheduled {new Date(detailItem.scheduled_at).toLocaleString()}
							{/if}
						</p>
					</div>
				{:else}
					<p class="text-muted-foreground">No matching draft in the content list for this id.</p>
				{/if}
			</div>
		</div>
	</div>
{/if}
