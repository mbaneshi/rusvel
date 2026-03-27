<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import RulesTab from '$lib/components/department/RulesTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<RulesTab dept={dept.id} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
