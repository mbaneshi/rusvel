<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import '@xterm/xterm/css/xterm.css';

	let termEl: HTMLDivElement;
	let term: Terminal | null = null;
	let fitAddon: FitAddon | null = null;
	let ws: WebSocket | null = null;
	let connected = $state(false);
	let error = $state('');

	function getWsUrl(): string {
		const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const host = window.location.hostname;
		// Use API port 3000 in dev (SvelteKit on 5173), same host in prod
		const port = window.location.port === '5173' ? '3000' : window.location.port;
		return `${proto}//${host}:${port}/api/terminal/ws`;
	}

	function connect() {
		error = '';
		const url = getWsUrl();
		ws = new WebSocket(url);
		ws.binaryType = 'arraybuffer';

		ws.onopen = () => {
			connected = true;
			term?.focus();
		};

		ws.onmessage = (ev: MessageEvent) => {
			if (ev.data instanceof ArrayBuffer) {
				term?.write(new Uint8Array(ev.data));
			} else if (typeof ev.data === 'string') {
				term?.write(ev.data);
			}
		};

		ws.onclose = () => {
			connected = false;
			term?.write('\r\n\x1b[90m[disconnected]\x1b[0m\r\n');
		};

		ws.onerror = () => {
			error = 'WebSocket connection failed';
			connected = false;
		};
	}

	function disconnect() {
		ws?.close();
		ws = null;
		connected = false;
	}

	onMount(() => {
		term = new Terminal({
			cursorBlink: true,
			fontSize: 14,
			fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", Menlo, monospace',
			theme: {
				background: '#09090b',
				foreground: '#fafafa',
				cursor: '#fafafa',
				selectionBackground: '#27272a',
				black: '#09090b',
				red: '#ef4444',
				green: '#22c55e',
				yellow: '#eab308',
				blue: '#3b82f6',
				magenta: '#a855f7',
				cyan: '#06b6d4',
				white: '#fafafa'
			}
		});

		fitAddon = new FitAddon();
		term.loadAddon(fitAddon);
		term.open(termEl);
		fitAddon.fit();

		term.onData((data: string) => {
			if (ws?.readyState === WebSocket.OPEN) {
				ws.send(data);
			}
		});

		const resizeObserver = new ResizeObserver(() => {
			fitAddon?.fit();
		});
		resizeObserver.observe(termEl);

		connect();

		return () => {
			resizeObserver.disconnect();
		};
	});

	onDestroy(() => {
		disconnect();
		term?.dispose();
	});
</script>

<svelte:head>
	<title>Terminal | RUSVEL</title>
</svelte:head>

<div class="flex h-[calc(100vh-3.5rem)] flex-col">
	<!-- Toolbar -->
	<div class="flex items-center justify-between border-b border-border px-4 py-2">
		<div class="flex items-center gap-3">
			<h1 class="text-sm font-semibold">Terminal</h1>
			<span
				class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium {connected
					? 'bg-green-500/10 text-green-500'
					: 'bg-muted text-muted-foreground'}"
			>
				<span
					class="h-1.5 w-1.5 rounded-full {connected ? 'bg-green-500' : 'bg-muted-foreground'}"
				></span>
				{connected ? 'Connected' : 'Disconnected'}
			</span>
		</div>
		<div class="flex gap-2">
			{#if connected}
				<button
					type="button"
					onclick={disconnect}
					class="rounded-md border border-border px-3 py-1 text-xs text-muted-foreground hover:text-foreground"
				>
					Disconnect
				</button>
			{:else}
				<button
					type="button"
					onclick={connect}
					class="rounded-md border border-border px-3 py-1 text-xs text-muted-foreground hover:text-foreground"
				>
					Reconnect
				</button>
			{/if}
		</div>
	</div>

	{#if error}
		<div class="border-b border-red-500/20 bg-red-500/5 px-4 py-2 text-xs text-red-500">
			{error}
		</div>
	{/if}

	<!-- Terminal container -->
	<div class="min-h-0 flex-1 bg-[#09090b] p-1" bind:this={termEl}></div>
</div>
