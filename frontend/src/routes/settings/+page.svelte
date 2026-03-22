<script lang="ts">
	import { checkHealth } from '$lib/api';
	let health = $state('checking...');
	let version = $state('0.1.0');

	async function check() {
		try {
			const res = await checkHealth();
			health = res.status === 'ok' ? 'Connected' : 'Error';
		} catch {
			health = 'Disconnected';
		}
	}

	check();
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold text-gray-100">Settings</h1>

	<div class="max-w-2xl space-y-6">
		<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">System</h3>
			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">Version</span>
					<span class="text-sm text-gray-200">{version}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">API Status</span>
					<span class="text-sm {health === 'Connected' ? 'text-green-400' : 'text-red-400'}">{health}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">LLM Provider</span>
					<span class="text-sm text-gray-200">Claude CLI (Max subscription)</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm text-gray-400">Database</span>
					<span class="text-sm text-gray-200">SQLite WAL (~/.rusvel/rusvel.db)</span>
				</div>
			</div>
		</div>

		<div class="rounded-xl border border-gray-800 bg-gray-900 p-5">
			<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-400">Engines</h3>
			<div class="space-y-2">
				{#each [
					{ name: 'Forge', tests: 15, status: 'Active' },
					{ name: 'Code', tests: 6, status: 'Built' },
					{ name: 'Harvest', tests: 12, status: 'Built' },
					{ name: 'Content', tests: 7, status: 'Built' },
					{ name: 'GoToMarket', tests: 5, status: 'Built' }
				] as engine}
					<div class="flex items-center justify-between rounded-lg bg-gray-800/50 px-3 py-2">
						<span class="text-sm text-gray-200">{engine.name}</span>
						<div class="flex items-center gap-3">
							<span class="text-xs text-gray-500">{engine.tests} tests</span>
							<span class="rounded-full px-2 py-0.5 text-xs {engine.status === 'Active' ? 'bg-green-900/50 text-green-300' : 'bg-gray-700 text-gray-400'}">
								{engine.status}
							</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
</div>
