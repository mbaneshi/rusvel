<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { marked } from 'marked';
	import { streamChat, getConversations, getChatHistory } from '$lib/api';
	import type { Conversation } from '$lib/api';
	import ChatTopBar from './ChatTopBar.svelte';

	interface DisplayMessage {
		role: 'user' | 'assistant' | 'system';
		content: string;
		streaming?: boolean;
	}

	let messages: DisplayMessage[] = $state([]);
	let conversations: Conversation[] = $state([]);
	let conversationId: string | undefined = $state(undefined);
	let inputText = $state('');
	let sending = $state(false);
	let error = $state('');
	let messagesContainer: HTMLDivElement | undefined = $state(undefined);
	let textareaEl: HTMLTextAreaElement | undefined = $state(undefined);
	let showHistory = $state(false);

	marked.setOptions({ breaks: true, gfm: true });

	function renderMarkdown(text: string): string {
		if (!text) return '';
		return marked.parse(text) as string;
	}

	onMount(async () => {
		try {
			conversations = await getConversations();
		} catch {
			// No conversations yet
		}
	});

	async function scrollToBottom() {
		await tick();
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	}

	function autoResize() {
		if (textareaEl) {
			textareaEl.style.height = 'auto';
			textareaEl.style.height = Math.min(textareaEl.scrollHeight, 120) + 'px';
		}
	}

	async function loadConversation(id: string) {
		try {
			const history = await getChatHistory(id);
			messages = history.map((m) => ({ role: m.role, content: m.content }));
			conversationId = id;
			showHistory = false;
			await scrollToBottom();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		}
	}

	function newConversation() {
		messages = [];
		conversationId = undefined;
		inputText = '';
		error = '';
		showHistory = false;
	}

	async function sendMessage() {
		const text = inputText.trim();
		if (!text || sending) return;

		inputText = '';
		if (textareaEl) textareaEl.style.height = 'auto';
		sending = true;
		error = '';

		messages = [...messages, { role: 'user', content: text }];
		await scrollToBottom();

		messages = [...messages, { role: 'assistant', content: '', streaming: true }];
		await scrollToBottom();

		try {
			await streamChat(
				text,
				conversationId,
				(deltaText, convId) => {
					conversationId = convId;
					const last = messages[messages.length - 1];
					if (last?.role === 'assistant') {
						messages = [
							...messages.slice(0, -1),
							{ ...last, content: last.content + deltaText }
						];
					}
					scrollToBottom();
				},
				(fullText, convId) => {
					conversationId = convId;
					messages = [
						...messages.slice(0, -1),
						{ role: 'assistant', content: fullText, streaming: false }
					];
					sending = false;
					getConversations().then((c) => (conversations = c)).catch(() => {});
				},
				(msg) => {
					error = msg;
					if (messages[messages.length - 1]?.content === '') {
						messages = messages.slice(0, -1);
					}
					sending = false;
				}
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Send failed';
			if (messages[messages.length - 1]?.content === '') {
				messages = messages.slice(0, -1);
			}
			sending = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			sendMessage();
		}
	}
</script>

<div class="flex h-full flex-col border-l border-[var(--r-border-default)] bg-[var(--r-bg-base)]">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-[var(--r-border-default)] px-3 py-2">
		<div class="flex items-center gap-2">
			<div class="flex h-6 w-6 items-center justify-center rounded-md bg-[var(--r-brand-default)] text-[10px] font-bold text-white">R</div>
			<span class="text-sm font-medium text-[var(--r-fg-default)]">Assistant</span>
		</div>
		<div class="flex items-center gap-1">
			<button
				onclick={() => (showHistory = !showHistory)}
				title="Conversation history"
				class="rounded-md p-1 text-[var(--r-fg-subtle)] hover:bg-[var(--r-bg-raised)] hover:text-[var(--r-fg-default)]"
			>
				<svg class="h-4 w-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<path d="M8 3.5V8L10.5 10.5" stroke-linecap="round" /><circle cx="8" cy="8" r="5.5" />
				</svg>
			</button>
			<button
				onclick={newConversation}
				title="New conversation"
				class="rounded-md p-1 text-[var(--r-fg-subtle)] hover:bg-[var(--r-bg-raised)] hover:text-[var(--r-fg-default)]"
			>
				<svg class="h-4 w-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<path d="M8 3v10M3 8h10" stroke-linecap="round" />
				</svg>
			</button>
		</div>
	</div>

	<!-- Top bar: Model, Effort, Tools -->
	<ChatTopBar />

	<!-- History dropdown -->
	{#if showHistory}
		<div class="max-h-48 overflow-y-auto border-b border-[var(--r-border-default)] bg-[var(--r-bg-surface)]">
			{#if conversations.length === 0}
				<p class="p-3 text-center text-xs text-[var(--r-fg-subtle)]">No conversations</p>
			{:else}
				{#each conversations as conv}
					<button
						onclick={() => loadConversation(conv.id)}
						class="w-full px-3 py-2 text-left text-xs transition-colors hover:bg-[var(--r-bg-raised)]
							{conversationId === conv.id ? 'bg-[var(--r-bg-raised)] text-[var(--r-fg-default)]' : 'text-[var(--r-fg-muted)]'}"
					>
						<p class="truncate">{conv.title}</p>
						<p class="text-[var(--r-fg-subtle)]">{conv.message_count} msgs</p>
					</button>
				{/each}
			{/if}
		</div>
	{/if}

	<!-- Messages -->
	<div bind:this={messagesContainer} class="flex-1 overflow-y-auto">
		{#if messages.length === 0}
			<div class="flex h-full items-center justify-center p-4">
				<div class="text-center">
					<div class="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--r-brand-default)]/20">
						<span class="text-lg font-bold text-brand-400">R</span>
					</div>
					<p class="text-xs text-[var(--r-fg-muted)]">Ask anything. I know your profile, products, and goals.</p>
				</div>
			</div>
		{:else}
			<div class="space-y-3 p-3">
				{#each messages as msg}
					{@const isUser = msg.role === 'user'}
					<div class="flex gap-2 {isUser ? 'justify-end' : 'justify-start'}">
						{#if !isUser}
							<div class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-md bg-[var(--r-brand-default)]/30 text-[10px] font-bold text-brand-300">R</div>
						{/if}
						<div class="max-w-[85%] rounded-xl px-3 py-2 text-xs leading-relaxed
							{isUser ? 'bg-[var(--r-brand-default)] text-white rounded-br-sm' : 'bg-[var(--r-bg-raised)] text-[var(--r-fg-default)] rounded-bl-sm'}">
							{#if isUser}
								<p class="whitespace-pre-wrap">{msg.content}</p>
							{:else if msg.streaming && !msg.content}
								<div class="flex items-center gap-1">
									<div class="h-1.5 w-1.5 animate-bounce rounded-full bg-brand-400" style="animation-delay: 0ms"></div>
									<div class="h-1.5 w-1.5 animate-bounce rounded-full bg-brand-400" style="animation-delay: 150ms"></div>
									<div class="h-1.5 w-1.5 animate-bounce rounded-full bg-brand-400" style="animation-delay: 300ms"></div>
								</div>
							{:else}
								<div class="prose prose-xs prose-invert max-w-none prose-headings:text-[var(--r-fg-default)] prose-headings:text-xs prose-p:text-[var(--r-fg-default)] prose-strong:text-[var(--r-fg-default)] prose-code:text-brand-300 prose-code:text-[10px] prose-pre:text-[10px] prose-li:text-[var(--r-fg-muted)]">
									{@html renderMarkdown(msg.content)}
								</div>
								{#if msg.streaming}
									<span class="inline-block h-3 w-0.5 animate-pulse bg-brand-400"></span>
								{/if}
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Error -->
	{#if error}
		<div class="mx-2 mb-1 rounded-md bg-danger-900/30 px-2 py-1 text-[10px] text-danger-400">
			{error}
			<button onclick={() => (error = '')} class="ml-1 underline">dismiss</button>
		</div>
	{/if}

	<!-- Input -->
	<div class="border-t border-[var(--r-border-default)] p-2">
		<div class="flex items-end gap-2">
			<textarea
				bind:this={textareaEl}
				bind:value={inputText}
				onkeydown={handleKeydown}
				oninput={autoResize}
				placeholder="Ask RUSVEL..."
				rows="1"
				disabled={sending}
				class="flex-1 resize-none rounded-lg border border-[var(--r-border-default)] bg-[var(--r-bg-raised)] px-3 py-2 text-xs text-[var(--r-fg-default)] placeholder-[var(--r-fg-subtle)] focus:border-[var(--r-border-brand)] focus:outline-none disabled:opacity-50"
			></textarea>
			<button
				onclick={sendMessage}
				disabled={sending || !inputText.trim()}
				class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg bg-[var(--r-brand-default)] transition-colors hover:bg-[var(--r-brand-hover)] disabled:opacity-30"
			>
				{#if sending}
					<div class="h-3 w-3 animate-spin rounded-full border-2 border-white/30 border-t-white"></div>
				{:else}
					<svg class="h-3.5 w-3.5 text-white" viewBox="0 0 20 20" fill="currentColor">
						<path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
					</svg>
				{/if}
			</button>
		</div>
	</div>
</div>
