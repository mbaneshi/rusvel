<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import EngineTab from '$lib/components/department/EngineTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<EngineTab dept={dept.id} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
