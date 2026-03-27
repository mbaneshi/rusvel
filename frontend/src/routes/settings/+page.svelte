<script lang="ts">
	import { checkHealth, getPendingApprovals, approveJob, rejectJob, type Job } from '$lib/api';
	import { refreshPendingApprovalCount } from '$lib/stores';
	import { toast } from 'svelte-sonner';
	let health = $state('checking...');
	let version = $state('0.1.0');

	let pendingJobs = $state<Job[]>([]);
	let approvalsLoading = $state(true);
	let approvalsError = $state('');
	let actionInFlight = $state<string | null>(null);

	async function check() {
		try {
			const res = await checkHealth();
			health = res.status === 'ok' ? 'Connected' : 'Error';
		} catch {
			health = 'Disconnected';
		}
	}

	async function loadApprovals() {
		approvalsLoading = true;
		approvalsError = '';
		try {
			pendingJobs = await getPendingApprovals();
		} catch (e) {
			approvalsError = e instanceof Error ? e.message : 'Failed to load approvals';
		} finally {
			approvalsLoading = false;
		}
	}

	async function handleApprove(id: string) {
		actionInFlight = id;
		try {
			await approveJob(id);
			pendingJobs = pendingJobs.filter((j) => j.id !== id);
			await refreshPendingApprovalCount();
			toast.success('Job approved');
		} catch (e) {
			approvalsError = e instanceof Error ? e.message : 'Failed to approve job';
			toast.error(approvalsError);
		} finally {
			actionInFlight = null;
		}
	}

	async function handleReject(id: string) {
		actionInFlight = id;
		try {
			await rejectJob(id);
			pendingJobs = pendingJobs.filter((j) => j.id !== id);
			await refreshPendingApprovalCount();
			toast.success('Job rejected');
		} catch (e) {
			approvalsError = e instanceof Error ? e.message : 'Failed to reject job';
			toast.error(approvalsError);
		} finally {
			actionInFlight = null;
		}
	}

	function formatKind(kind: Job['kind']): string {
		if (typeof kind === 'string') return kind;
		if (typeof kind === 'object' && kind !== null) {
			const key = Object.keys(kind)[0];
			return key ?? JSON.stringify(kind);
		}
		return String(kind);
	}

	check();
	loadApprovals();
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold text-gray-100">Settings</h1>

	<p class="mb-4 text-sm text-gray-400">
		<a href="/settings/spend" class="text-primary hover:underline">LLM spend dashboard</a>
	</p>

	<div class="max-w-2xl space-y-6">
		<!-- Approvals Section -->
		<div class="rounded-xl border border-amber-800/50 bg-gray-900 p-5">
			<div class="mb-4 flex items-center justify-between">
				<h3 class="text-sm font-semibold uppercase tracking-wider text-amber-400">
					Pending Approvals
				</h3>
				<button
					onclick={loadApprovals}
					class="rounded px-2 py-1 text-xs text-gray-400 transition hover:bg-gray-800 hover:text-gray-200"
				>
					Refresh
				</button>
			</div>

			{#if approvalsLoading}
				<p class="text-sm text-gray-500">Loading...</p>
			{:else if approvalsError}
				<p class="text-sm text-red-400">{approvalsError}</p>
			{:else if pendingJobs.length === 0}
				<p class="text-sm text-gray-500">No jobs awaiting approval.</p>
			{:else}
				<div class="space-y-3">
					{#each pendingJobs as job (job.id)}
						<div class="rounded-lg border border-gray-700 bg-gray-800/50 p-3">
							<div class="mb-2 flex items-start justify-between">
								<div>
									<span
										class="inline-block rounded bg-amber-900/50 px-2 py-0.5 text-xs font-medium text-amber-300"
									>
										{formatKind(job.kind)}
									</span>
									<span class="ml-2 text-xs text-gray-500" title={job.id}>
										{job.id.slice(0, 8)}...
									</span>
								</div>
								<span class="text-xs text-gray-500">
									retries: {job.retries}/{job.max_retries}
								</span>
							</div>

							{#if job.payload && typeof job.payload === 'object'}
								<pre
									class="mb-3 max-h-24 overflow-auto rounded bg-gray-900 p-2 text-xs text-gray-400">{JSON.stringify(
										job.payload,
										null,
										2
									)}</pre>
							{/if}

							<div class="flex gap-2">
								<button
									onclick={() => handleApprove(job.id)}
									disabled={actionInFlight === job.id}
									class="rounded bg-green-700 px-3 py-1 text-xs font-medium text-green-100 transition hover:bg-green-600 disabled:opacity-50"
								>
									{actionInFlight === job.id ? 'Processing...' : 'Approve'}
								</button>
								<button
									onclick={() => handleReject(job.id)}
									disabled={actionInFlight === job.id}
									class="rounded bg-red-800 px-3 py-1 text-xs font-medium text-red-100 transition hover:bg-red-700 disabled:opacity-50"
								>
									{actionInFlight === job.id ? 'Processing...' : 'Reject'}
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">System</h3>
			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">Version</span>
					<span class="text-sm text-gray-200">{version}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">API Status</span>
					<span class="text-sm {health === 'Connected' ? 'text-green-400' : 'text-red-400'}"
						>{health}</span
					>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">LLM Provider</span>
					<span class="text-sm text-gray-200">Claude CLI (Max subscription)</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">Database</span>
					<span class="text-sm text-gray-200">SQLite WAL (~/.rusvel/rusvel.db)</span>
				</div>
			</div>
		</div>

		<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">Engines</h3>
			<div class="space-y-2">
				{#each [{ name: 'Forge', tests: 15, status: 'Active' }, { name: 'Code', tests: 6, status: 'Built' }, { name: 'Harvest', tests: 12, status: 'Built' }, { name: 'Content', tests: 7, status: 'Built' }, { name: 'GoToMarket', tests: 5, status: 'Built' }] as engine}
					<div class="flex items-center justify-between rounded-lg bg-gray-800/50 px-3 py-2">
						<span class="text-sm text-gray-200">{engine.name}</span>
						<div class="flex items-center gap-3">
							<span class="text-xs text-gray-500">{engine.tests} tests</span>
							<span
								class="rounded-full px-2 py-0.5 text-xs {engine.status === 'Active'
									? 'bg-green-900/50 text-green-300'
									: 'bg-gray-700 text-gray-400'}"
							>
								{engine.status}
							</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
</div>
