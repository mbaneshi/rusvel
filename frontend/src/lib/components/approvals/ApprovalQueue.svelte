<script lang="ts">
	import { onMount } from 'svelte';
	import { getPendingApprovals, approveJob, rejectJob, type Job } from '$lib/api';
	import { refreshPendingApprovalCount } from '$lib/stores';
	import Button from '$lib/components/ui/Button.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import { toast } from 'svelte-sonner';

	let jobs: Job[] = $state([]);
	let loading = $state(true);
	let busyId = $state<string | null>(null);

	async function load() {
		loading = true;
		try {
			jobs = await getPendingApprovals();
			await refreshPendingApprovalCount();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load approvals');
			jobs = [];
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		load();
	});

	function formatPayload(payload: unknown): string {
		if (payload === null || payload === undefined) return '—';
		if (typeof payload === 'string') return payload;
		try {
			return JSON.stringify(payload, null, 2);
		} catch {
			return String(payload);
		}
	}

	async function approve(id: string) {
		busyId = id;
		try {
			await approveJob(id);
			jobs = jobs.filter((j) => j.id !== id);
			await refreshPendingApprovalCount();
			toast.success('Approved');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Approve failed');
		} finally {
			busyId = null;
		}
	}

	async function reject(id: string) {
		busyId = id;
		try {
			await rejectJob(id);
			jobs = jobs.filter((j) => j.id !== id);
			await refreshPendingApprovalCount();
			toast.success('Rejected');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Reject failed');
		} finally {
			busyId = null;
		}
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between gap-3">
		<p class="text-xs text-muted-foreground">
			Content publishing, outreach, and other gated work pauses here until you approve or reject.
		</p>
		<Button variant="secondary" size="sm" onclick={() => load()} disabled={loading}>
			{loading ? 'Refreshing…' : 'Refresh'}
		</Button>
	</div>

	{#if loading && jobs.length === 0}
		<div class="rounded-lg border border-border bg-card p-8 text-center text-sm text-muted-foreground">
			Loading queue…
		</div>
	{:else if jobs.length === 0}
		<EmptyState
			icon="check-circle"
			title="No pending approvals"
			description="When a job needs your sign-off, it will appear here and in department chat."
		/>
	{:else}
		<ul class="space-y-3">
			{#each jobs as job (job.id)}
				<li
					class="rounded-lg border border-border bg-card p-4 shadow-sm"
				>
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div class="min-w-0 flex-1">
							<div class="flex flex-wrap items-center gap-2">
								<span
									class="rounded-md bg-warning-500/15 px-2 py-0.5 text-[11px] font-medium text-warning-400"
								>
									{typeof job.kind === 'string' ? job.kind : JSON.stringify(job.kind)}
								</span>
								<span class="font-mono text-[10px] text-muted-foreground">{job.id}</span>
							</div>
							<p class="mt-1 text-[11px] text-muted-foreground">
								Session <span class="font-mono">{job.session_id}</span>
								<span class="mx-1">·</span>
								{job.status}
							</p>
						</div>
						<div class="flex shrink-0 gap-2">
							<Button
								variant="primary"
								size="sm"
								disabled={busyId !== null}
								loading={busyId === job.id}
								onclick={() => approve(job.id)}
							>
								Approve
							</Button>
							<Button
								variant="danger"
								size="sm"
								disabled={busyId !== null}
								loading={busyId === job.id}
								onclick={() => reject(job.id)}
							>
								Reject
							</Button>
						</div>
					</div>
					<pre
						class="mt-3 max-h-40 overflow-auto rounded-md bg-muted/40 p-3 font-mono text-[11px] leading-relaxed text-muted-foreground"
					>{formatPayload(job.payload)}</pre>
				</li>
			{/each}
		</ul>
	{/if}
</div>
