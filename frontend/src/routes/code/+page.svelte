<script lang="ts">
	import { activeSession } from '$lib/stores';
	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold text-gray-100">Code Intelligence</h1>

	{#if !currentSession}
		<div class="rounded-xl border border-gray-800 bg-gray-900 p-8 text-center">
			<p class="text-lg text-gray-400">No session selected</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-2 text-sm font-semibold uppercase tracking-wider text-gray-400">Parser</h3>
				<p class="text-sm text-gray-500">Rust source file parsing and AST analysis</p>
				<p class="mt-3 text-xs text-gray-600">Engine ready - CLI/API not yet wired</p>
			</div>
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-2 text-sm font-semibold uppercase tracking-wider text-gray-400">Dependency Graph</h3>
				<p class="text-sm text-gray-500">Module and crate dependency visualization</p>
				<p class="mt-3 text-xs text-gray-600">Engine ready - CLI/API not yet wired</p>
			</div>
			<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
				<h3 class="mb-2 text-sm font-semibold uppercase tracking-wider text-gray-400">Search</h3>
				<p class="text-sm text-gray-500">BM25 full-text search across codebase</p>
				<p class="mt-3 text-xs text-gray-600">Engine ready - CLI/API not yet wired</p>
			</div>
		</div>
		<p class="mt-6 text-sm text-gray-600">
			Code engine has 6 passing tests. Needs CLI commands and API endpoints to be exposed.
		</p>
	{/if}
</div>
