<script lang="ts">
	import DeptSectionScaffold from '$lib/components/department/DeptSectionScaffold.svelte';
	import DeptTerminalPane from '$lib/components/department/DeptTerminalPane.svelte';
	import { activeSession } from '$lib/stores';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));
</script>

<DeptSectionScaffold>
	{#snippet children({ dept })}
		<DeptTerminalPane dept={dept.id} sessionId={currentSession?.id ?? null} />
	{/snippet}
</DeptSectionScaffold>
