<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import SkillsTab from '$lib/components/department/SkillsTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept, deptHsl })}
		<div class="h-full overflow-auto">
			<SkillsTab dept={dept.id} {deptHsl} />
		</div>
	{/snippet}
</DeptSectionScaffold>
