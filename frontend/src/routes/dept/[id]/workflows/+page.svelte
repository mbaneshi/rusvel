<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import WorkflowsTab from '$lib/components/department/WorkflowsTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<WorkflowsTab dept={dept.id} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
