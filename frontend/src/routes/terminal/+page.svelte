<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DeptTerminal from '$lib/components/DeptTerminal.svelte';
	import { STANDALONE_TERMINAL_DEPT_ID } from '$lib/terminalConstants';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	let terminalPaneId = $state<string | null>(null);
	let terminalPaneForKey = $state<string | null>(null);
	let terminalLoading = $state(false);
	let terminalErr = $state('');

	function apiBase(): string {
		if (typeof window === 'undefined') return '';
		const { protocol, hostname, port } = window.location;
		const apiPort = port === '5173' ? '3000' : port;
		return `${protocol}//${hostname}${apiPort ? `:${apiPort}` : ''}`;
	}

	$effect(() => {
		const sessionId = currentSession?.id;
		if (!sessionId) {
			terminalPaneId = null;
			terminalPaneForKey = null;
			terminalErr = '';
			terminalLoading = false;
			return;
		}

		const key = `${sessionId}:${STANDALONE_TERMINAL_DEPT_ID}`;
		if (terminalPaneForKey === key && terminalPaneId) return;

		let cancelled = false;
		terminalLoading = true;
		terminalErr = '';
		const url = `${apiBase()}/api/terminal/dept/${encodeURIComponent(STANDALONE_TERMINAL_DEPT_ID)}?session_id=${encodeURIComponent(sessionId)}`;
		fetch(url)
			.then((r) => {
				if (!r.ok) return r.text().then((t) => Promise.reject(new Error(t || r.statusText)));
				return r.json();
			})
			.then((j) => {
				if (!cancelled && j.pane_id) {
					terminalPaneId = j.pane_id;
					terminalPaneForKey = key;
				}
			})
			.catch((e: unknown) => {
				if (!cancelled) terminalErr = e instanceof Error ? e.message : 'Failed to open terminal';
			})
			.finally(() => {
				if (!cancelled) terminalLoading = false;
			});
		return () => {
			cancelled = true;
		};
	});
</script>

<svelte:head>
	<title>Terminal | RUSVEL</title>
</svelte:head>

<div class="flex h-[calc(100vh-3.5rem)] flex-col">
	<div class="flex items-center justify-between border-b border-border px-4 py-2">
		<h1 class="text-sm font-semibold">Terminal</h1>
	</div>

	<div class="min-h-0 flex-1 p-2">
		{#if !currentSession?.id}
			<p class="text-xs text-muted-foreground">Select a session in the sidebar to use the terminal.</p>
		{:else if terminalLoading}
			<p class="text-xs text-muted-foreground">Starting terminal…</p>
		{:else if terminalErr}
			<p class="text-xs text-red-500">{terminalErr}</p>
		{:else if terminalPaneId}
			{#key terminalPaneId}
				<div class="h-full min-h-[320px]">
					<DeptTerminal paneId={terminalPaneId} />
				</div>
			{/key}
		{/if}
	</div>
</div>
