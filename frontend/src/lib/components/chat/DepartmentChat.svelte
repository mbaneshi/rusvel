<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { Streamdown } from 'svelte-streamdown';
	import { copy } from 'svelte-copy';
	import { toast } from 'svelte-sonner';
	import {
		streamDeptChat,
		streamHelp,
		getDeptConversations,
		getDeptChatHistory,
		getDeptConfig,
		updateDeptConfig,
		getModels,
		getTools
	} from '$lib/api';
	import type { Conversation, DepartmentConfig, ModelOption, ToolOption } from '$lib/api';
	import { onboarding, pendingCommand } from '$lib/stores';
	import { cached } from '$lib/cache';

	interface DisplayMessage {
		role: 'user' | 'assistant' | 'system';
		content: string;
		streaming?: boolean;
	}

	let {
		dept,
		title = 'Department',
		icon = 'D',
		suggestedPrompts = []
	}: { dept: string; title?: string; icon?: string; suggestedPrompts?: string[] } = $props();

	let messages: DisplayMessage[] = $state([]);
	let conversations: Conversation[] = $state([]);
	let conversationId: string | undefined = $state(undefined);
	let inputText = $state('');
	let sending = $state(false);
	let error = $state('');
	let messagesEl: HTMLDivElement | undefined = $state(undefined);
	let textareaEl: HTMLTextAreaElement | undefined = $state(undefined);
	let showHistory = $state(false);
	let showConfig = $state(false);

	// Config state
	let config: DepartmentConfig | null = $state(null);
	let models: ModelOption[] = $state([]);
	let tools: ToolOption[] = $state([]);

	// Streamdown handles markdown rendering with streaming support

	onMount(() => {
		// Load data
		Promise.all([
			getDeptConversations(dept),
			cached(`dept-config:${dept}`, () => getDeptConfig(dept)),
			cached('models', getModels),
			cached('tools', getTools)
		])
			.then(([convs, cfg, mdls, tls]) => {
				conversations = convs;
				config = cfg;
				models = mdls;
				tools = tls;
			})
			.catch(() => {
				/* defaults are fine */
			});

		textareaEl?.focus();
	});

	pendingCommand.subscribe((cmd) => {
		if (cmd) {
			inputText = cmd.prompt;
			send();
			pendingCommand.set(null);
		}
	});

	async function scroll() {
		await tick();
		if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
	}
	function autoResize() {
		if (textareaEl) {
			textareaEl.style.height = 'auto';
			textareaEl.style.height = Math.min(textareaEl.scrollHeight, 120) + 'px';
		}
	}

	async function loadConversation(id: string) {
		const history = await getDeptChatHistory(dept, id);
		messages = history.map((m) => ({ role: m.role, content: m.content }));
		conversationId = id;
		showHistory = false;
		await scroll();
	}

	function newChat() {
		messages = [];
		conversationId = undefined;
		inputText = '';
		error = '';
		showHistory = false;
	}

	async function send() {
		const text = inputText.trim();
		if (!text || sending) return;
		inputText = '';
		if (textareaEl) textareaEl.style.height = 'auto';
		sending = true;
		error = '';

		messages = [...messages, { role: 'user', content: text }];
		await scroll();
		messages = [...messages, { role: 'assistant', content: '', streaming: true }];
		await scroll();

		// Intercept /help commands — route to AI help endpoint
		if (text.startsWith('/help')) {
			const question = text.slice(5).trim() || 'How do I use RUSVEL?';
			try {
				await streamHelp(
					question,
					dept,
					(delta) => {
						const last = messages[messages.length - 1];
						if (last?.role === 'assistant')
							messages = [...messages.slice(0, -1), { ...last, content: last.content + delta }];
						scroll();
					},
					(full) => {
						messages = [
							...messages.slice(0, -1),
							{ role: 'assistant', content: full, streaming: false }
						];
						sending = false;
					},
					(msg) => {
						error = msg;
						if (messages[messages.length - 1]?.content === '') messages = messages.slice(0, -1);
						sending = false;
					}
				);
			} catch (e) {
				error = e instanceof Error ? e.message : 'Help failed';
				if (messages[messages.length - 1]?.content === '') messages = messages.slice(0, -1);
				sending = false;
			}
			return;
		}

		try {
			await streamDeptChat(
				dept,
				text,
				conversationId,
				(delta, convId) => {
					conversationId = convId;
					const last = messages[messages.length - 1];
					if (last?.role === 'assistant')
						messages = [...messages.slice(0, -1), { ...last, content: last.content + delta }];
					scroll();
				},
				(full, convId) => {
					conversationId = convId;
					messages = [
						...messages.slice(0, -1),
						{ role: 'assistant', content: full, streaming: false }
					];
					sending = false;
					onboarding.complete('deptChatUsed');
					getDeptConversations(dept)
						.then((c) => (conversations = c))
						.catch(() => {});
				},
				(msg) => {
					error = msg;
					if (messages[messages.length - 1]?.content === '') messages = messages.slice(0, -1);
					sending = false;
				}
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Send failed';
			if (messages[messages.length - 1]?.content === '') messages = messages.slice(0, -1);
			sending = false;
		}
	}

	function keydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			send();
		}
	}

	async function saveConfig() {
		if (!config) return;
		try {
			config = await updateDeptConfig(dept, config);
		} catch {
			/* silent */
		}
	}

	function setModel(e: globalThis.Event) {
		if (config) {
			config.model = (e.target as HTMLSelectElement).value;
			saveConfig();
		}
	}
	function setEffort(level: string) {
		if (config) {
			config.effort = level;
			saveConfig();
		}
	}
	function toggleTool(name: string) {
		if (!config) return;
		const idx = config.disallowed_tools.indexOf(name);
		config.disallowed_tools =
			idx >= 0
				? config.disallowed_tools.filter((t) => t !== name)
				: [...config.disallowed_tools, name];
		saveConfig();
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-border bg-card px-3 py-2">
		<div class="flex items-center gap-2">
			<div
				class="flex h-6 w-6 items-center justify-center rounded-md bg-chart-2/30 text-[10px] font-bold text-chart-2"
			>
				{icon}
			</div>
			<span class="text-sm font-medium text-foreground">{title}</span>
		</div>
		<div class="flex items-center gap-1">
			<button
				onclick={() => (showConfig = !showConfig)}
				title="Settings"
				class="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground {showConfig
					? 'text-foreground bg-accent'
					: ''}"
			>
				<svg
					class="h-4 w-4"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					><path
						d="M6.5 1.5h3l.5 2 1.5.7 1.8-1 2.1 2.1-1 1.8.7 1.5 2 .5v3l-2 .5-.7 1.5 1 1.8-2.1 2.1-1.8-1-1.5.7-.5 2h-3l-.5-2-1.5-.7-1.8 1-2.1-2.1 1-1.8-.7-1.5-2-.5v-3l2-.5.7-1.5-1-1.8L4.2 2.2l1.8 1L7.5 2.5z"
					/><circle cx="8" cy="8" r="2" /></svg
				>
			</button>
			<button
				onclick={() => (showHistory = !showHistory)}
				title="History"
				class="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
			>
				<svg
					class="h-4 w-4"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					><path d="M8 3.5V8L10.5 10.5" stroke-linecap="round" /><circle
						cx="8"
						cy="8"
						r="5.5"
					/></svg
				>
			</button>
			<button
				onclick={newChat}
				title="New chat"
				class="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
			>
				<svg
					class="h-4 w-4"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"><path d="M8 3v10M3 8h10" stroke-linecap="round" /></svg
				>
			</button>
		</div>
	</div>

	<!-- Config panel -->
	{#if showConfig && config}
		<div class="border-b border-border bg-card px-3 py-2 space-y-2">
			<div class="flex items-center gap-2">
				<span class="text-xs text-muted-foreground">Model</span>
				<select
					value={config.model}
					onchange={setModel}
					class="rounded-md border border-border bg-secondary px-2 py-0.5 text-xs text-foreground focus:outline-none"
				>
					{#each models as m}<option value={m.value}>{m.label}</option>{/each}
				</select>
				<span class="text-xs text-muted-foreground">Effort</span>
				<div class="flex rounded-md border border-border bg-secondary">
					{#each ['low', 'medium', 'high', 'max'] as level}
						<button
							onclick={() => setEffort(level)}
							class="px-1.5 py-0.5 text-[10px] {config.effort === level
								? 'bg-primary text-primary-foreground'
								: 'text-muted-foreground'}">{level}</button
						>
					{/each}
				</div>
			</div>
			<div class="flex flex-wrap gap-1">
				{#each tools as tool}
					{@const enabled = !config.disallowed_tools.includes(tool.name)}
					<button
						onclick={() => toggleTool(tool.name)}
						class="rounded px-1.5 py-0.5 text-[10px] border {enabled
							? 'border-primary/50 text-primary bg-primary/10'
							: 'border-border text-muted-foreground line-through opacity-50'}"
					>
						{tool.name}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- History dropdown -->
	{#if showHistory}
		<div class="max-h-40 overflow-y-auto border-b border-border bg-card">
			{#each conversations as conv}
				<button
					onclick={() => loadConversation(conv.id)}
					class="w-full px-3 py-1.5 text-left text-xs hover:bg-accent {conversationId === conv.id
						? 'bg-accent'
						: 'text-muted-foreground'}"
				>
					<p class="truncate">{conv.title}</p>
				</button>
			{/each}
			{#if conversations.length === 0}<p class="p-3 text-center text-xs text-muted-foreground">
					No conversations
				</p>{/if}
		</div>
	{/if}

	<!-- Messages -->
	<div bind:this={messagesEl} class="flex-1 overflow-y-auto">
		{#if messages.length === 0}
			<div class="flex h-full items-center justify-center p-4">
				<div class="text-center">
					<div
						class="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-xl bg-chart-2/20"
					>
						<span class="text-xl font-bold text-chart-2">{icon}</span>
					</div>
					<p class="text-sm font-medium text-foreground">{title}</p>
					<p class="mt-1 text-xs text-muted-foreground">
						Ready to work. Ask anything or try a suggestion below.
					</p>
					{#if suggestedPrompts.length > 0}
						<div class="mt-4 flex flex-wrap justify-center gap-1.5">
							{#each suggestedPrompts as prompt}
								<button
									onclick={() => {
										inputText = prompt;
										send();
									}}
									class="rounded-full border border-border bg-secondary px-2.5 py-1 text-[11px] text-muted-foreground hover:border-chart-2/40 hover:text-foreground transition-colors"
								>
									{prompt}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{:else}
			<div class="space-y-2 p-3">
				{#each messages as msg}
					{@const isUser = msg.role === 'user'}
					<div class="flex gap-2 {isUser ? 'justify-end' : 'justify-start'}">
						{#if !isUser}
							<div
								class="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-md bg-chart-2/30 text-[9px] font-bold text-chart-2"
							>
								{icon}
							</div>
						{/if}
						<div
							class="max-w-[85%] rounded-xl px-3 py-2 text-xs leading-relaxed {isUser
								? 'bg-primary text-primary-foreground rounded-br-sm'
								: 'bg-secondary text-foreground rounded-bl-sm relative group'}"
						>
							{#if isUser}
								<p class="whitespace-pre-wrap">{msg.content}</p>
							{:else if msg.streaming && !msg.content}
								<div class="flex items-center gap-1">
									<div class="h-1.5 w-1.5 animate-bounce rounded-full bg-chart-2"></div>
									<div
										class="h-1.5 w-1.5 animate-bounce rounded-full bg-chart-2"
										style="animation-delay:150ms"
									></div>
									<div
										class="h-1.5 w-1.5 animate-bounce rounded-full bg-chart-2"
										style="animation-delay:300ms"
									></div>
								</div>
							{:else}
								<div class="max-w-none text-xs">
									<Streamdown
										content={msg.content}
										parseIncompleteMarkdown={!!msg.streaming}
										baseTheme="shadcn"
										animation={{
											enabled: !!msg.streaming,
											type: 'blur',
											duration: 300,
											tokenize: 'word'
										}}
									/>
								</div>
								{#if msg.streaming}<span class="inline-block h-3 w-0.5 animate-pulse bg-chart-2"
									></span>{/if}
							{/if}
							{#if !isUser && msg.content && !msg.streaming}
								<button
									use:copy={msg.content}
									onclick={() => toast.success('Copied')}
									class="absolute top-1 right-1 hidden group-hover:flex h-5 w-5 items-center justify-center rounded bg-secondary text-muted-foreground hover:text-foreground text-[10px]"
									title="Copy message"
								>
									<svg
										class="h-3 w-3"
										viewBox="0 0 16 16"
										fill="none"
										stroke="currentColor"
										stroke-width="1.5"
										><rect x="5" y="5" width="8" height="8" rx="1" /><path d="M3 11V3h8" /></svg
									>
								</button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	{#if error}
		<div class="mx-2 mb-1 rounded-md bg-destructive/10 px-2 py-1 text-[10px] text-destructive">
			{error} <button onclick={() => (error = '')} class="underline">dismiss</button>
		</div>
	{/if}

	<!-- Input -->
	<div class="border-t border-border p-2">
		<div class="flex items-end gap-2">
			<textarea
				bind:this={textareaEl}
				bind:value={inputText}
				onkeydown={keydown}
				oninput={autoResize}
				placeholder="Ask {title}..."
				rows="1"
				disabled={sending}
				class="flex-1 resize-none rounded-lg border border-input bg-background px-3 py-2 text-xs text-foreground placeholder-muted-foreground focus:outline-2 focus:outline-offset-2 focus:outline-ring disabled:opacity-50"
			></textarea>
			<button
				onclick={send}
				disabled={sending || !inputText.trim()}
				class="flex h-7 w-7 items-center justify-center rounded-lg bg-primary hover:bg-primary/90 disabled:opacity-30"
			>
				{#if sending}<div
						class="h-3 w-3 animate-spin rounded-full border-2 border-primary-foreground/30 border-t-primary-foreground"
					></div>
				{:else}<svg class="h-3 w-3 text-primary-foreground" viewBox="0 0 20 20" fill="currentColor"
						><path
							d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z"
						/></svg
					>{/if}
			</button>
		</div>
	</div>
</div>
