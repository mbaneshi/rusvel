<script lang="ts">
	import ApprovalQueue from '$lib/components/approvals/ApprovalQueue.svelte';
</script>

<div class="h-full overflow-y-auto">
	<div class="mx-auto max-w-3xl p-6">
		<h1 class="text-2xl font-semibold tracking-tight text-foreground">Approvals</h1>
		<p class="mt-1 text-sm text-muted-foreground">
			Human-in-the-loop queue (ADR-008). Approve or reject jobs that are waiting for you.
		</p>
		<div class="mt-8">
			<ApprovalQueue />
		</div>
	</div>
</div>
