<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import AgentsTab from '$lib/components/department/AgentsTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<AgentsTab dept={dept.id} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
