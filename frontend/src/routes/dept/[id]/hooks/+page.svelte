<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import HooksTab from '$lib/components/department/HooksTab.svelte';
</script>

<DeptSectionScaffold>
	{#snippet children({ dept })}
		<div class="h-full overflow-auto">
			<HooksTab dept={dept.id} />
		</div>
	{/snippet}
</DeptSectionScaffold>
