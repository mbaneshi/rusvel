<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import ActionsTab from '$lib/components/department/ActionsTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<ActionsTab dept={dept.id} quickActions={dept.quick_actions} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
