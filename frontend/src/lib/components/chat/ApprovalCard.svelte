<script lang="ts">
	let {
		jobId,
		jobKind,
		payload,
		onApprove,
		onReject
	}: {
		jobId: string;
		jobKind: string;
		payload: Record<string, unknown>;
		onApprove: (jobId: string) => void;
		onReject: (jobId: string) => void;
	} = $props();

	let decided = $state(false);
	let decision = $state<'approved' | 'rejected' | null>(null);

	function approve() {
		decided = true;
		decision = 'approved';
		onApprove(jobId);
	}

	function reject() {
		decided = true;
		decision = 'rejected';
		onReject(jobId);
	}
</script>

<div class="border-l-2 border-yellow-500 bg-zinc-900/50 rounded-r-md px-3 py-2 my-2 font-mono text-sm">
	<div class="flex items-center gap-2 text-yellow-400 text-xs mb-2">
		<span>&#x23F3;</span>
		<span>Awaiting approval: {jobKind}</span>
	</div>
	<pre class="bg-zinc-950 p-2 rounded overflow-x-auto max-h-32 text-xs text-zinc-400 mb-2">{JSON.stringify(payload, null, 2)}</pre>
	{#if decided}
		<div class="text-xs {decision === 'approved' ? 'text-green-400' : 'text-red-400'}">
			{decision === 'approved' ? 'Approved' : 'Rejected'}
		</div>
	{:else}
		<div class="flex gap-2">
			<button
				onclick={approve}
				class="rounded px-3 py-1 text-xs font-medium bg-green-600 hover:bg-green-500 text-white transition-colors"
			>
				Approve
			</button>
			<button
				onclick={reject}
				class="rounded px-3 py-1 text-xs font-medium bg-red-600 hover:bg-red-500 text-white transition-colors"
			>
				Reject
			</button>
		</div>
	{/if}
</div>
