<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import McpTab from '$lib/components/department/McpTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept })}
		<div class="h-full overflow-auto">
			<McpTab dept={dept.id} />
		</div>
	{/snippet}
</DeptSectionScaffold>
