<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { marked } from 'marked';
	import { streamChat, getConversations, getChatHistory } from '$lib/api';
	import type { Conversation } from '$lib/api';
	import ChatTopBar from '$lib/components/chat/ChatTopBar.svelte';

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

	// Configure marked for safe rendering
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
		textareaEl?.focus();
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
			textareaEl.style.height = Math.min(textareaEl.scrollHeight, 200) + 'px';
		}
	}

	async function loadConversation(id: string) {
		try {
			const history = await getChatHistory(id);
			messages = history.map((m) => ({ role: m.role, content: m.content }));
			conversationId = id;
			await scrollToBottom();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load conversation';
		}
	}

	function newConversation() {
		messages = [];
		conversationId = undefined;
		inputText = '';
		error = '';
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
			error = e instanceof Error ? e.message : 'Failed to send message';
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

	function formatTime(conv: Conversation): string {
		try {
			const d = new Date(conv.updated_at);
			const now = new Date();
			if (d.toDateString() === now.toDateString()) {
				return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
			}
			return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
		} catch {
			return '';
		}
	}
</script>

<div class="flex h-full">
	<!-- Conversation sidebar -->
	<div class="flex w-60 flex-shrink-0 flex-col border-r border-gray-800 bg-gray-900/50">
		<div class="border-b border-gray-800 p-3">
			<button
				onclick={newConversation}
				class="flex w-full items-center justify-center gap-2 rounded-lg bg-indigo-600 px-3 py-2.5 text-sm font-medium transition-colors hover:bg-indigo-500"
			>
				<span class="text-lg leading-none">+</span> New Chat
			</button>
		</div>
		<div class="flex-1 overflow-y-auto p-2">
			{#if conversations.length === 0}
				<p class="p-3 text-center text-xs text-gray-600">No conversations yet.<br />Start chatting below.</p>
			{:else}
				{#each conversations as conv}
					<button
						onclick={() => loadConversation(conv.id)}
						class="mb-1 w-full rounded-lg px-3 py-2.5 text-left transition-colors
							{conversationId === conv.id
							? 'bg-indigo-600/15 border border-indigo-500/30 text-gray-100'
							: 'border border-transparent text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
					>
						<p class="truncate text-sm">{conv.title}</p>
						<div class="mt-0.5 flex items-center gap-2 text-xs text-gray-600">
							<span>{conv.message_count} msgs</span>
							<span>{formatTime(conv)}</span>
						</div>
					</button>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Chat area -->
	<div class="flex flex-1 flex-col">
		<!-- Top bar: Model, Effort, Tools -->
		<ChatTopBar />

		<!-- Messages -->
		<div bind:this={messagesContainer} class="flex-1 overflow-y-auto">
			{#if messages.length === 0}
				<div class="flex h-full items-center justify-center p-6">
					<div class="text-center">
						<div class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-indigo-600/30 to-purple-600/20">
							<span class="text-3xl font-bold text-indigo-400">R</span>
						</div>
						<h2 class="text-xl font-semibold text-gray-200">RUSVEL Assistant</h2>
						<p class="mt-2 max-w-sm text-sm leading-relaxed text-gray-500">
							Your AI companion that knows your products, skills, and mission. Plan your day, draft content, strategize, or just think out loud.
						</p>
						<div class="mt-6 flex flex-wrap justify-center gap-2">
							{#each ['Plan my day', 'Draft a blog post', 'Review my goals', 'What should I focus on?'] as suggestion}
								<button
									onclick={() => { inputText = suggestion; sendMessage(); }}
									class="rounded-full border border-gray-700 bg-gray-800/50 px-3 py-1.5 text-xs text-gray-400 transition-colors hover:border-indigo-500/50 hover:text-gray-200"
								>
									{suggestion}
								</button>
							{/each}
						</div>
					</div>
				</div>
			{:else}
				<div class="mx-auto max-w-3xl space-y-1 p-6">
					{#each messages as msg, i}
						{@const isUser = msg.role === 'user'}
						{@const showAvatar = i === 0 || messages[i - 1]?.role !== msg.role}
						<div class="flex gap-3 {isUser ? 'justify-end' : 'justify-start'} {showAvatar ? 'mt-4' : 'mt-1'}">
							{#if !isUser && showAvatar}
								<div class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-indigo-600/40 to-purple-600/30 text-xs font-bold text-indigo-300">
									R
								</div>
							{:else if !isUser}
								<div class="w-8 flex-shrink-0"></div>
							{/if}

							{#if isUser}
								<div class="max-w-[75%] rounded-2xl rounded-br-md bg-indigo-600 px-4 py-2.5 text-sm text-white">
									<p class="whitespace-pre-wrap">{msg.content}</p>
								</div>
								{#if showAvatar}
									<div class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg bg-gray-700 text-xs font-bold text-gray-300">
										M
									</div>
								{:else}
									<div class="w-8 flex-shrink-0"></div>
								{/if}
							{:else}
								<div class="max-w-[85%] rounded-2xl rounded-bl-md bg-gray-800/80 px-4 py-3 text-sm text-gray-200">
									{#if msg.streaming && !msg.content}
										<div class="flex items-center gap-1.5">
											<div class="h-2 w-2 animate-bounce rounded-full bg-indigo-400" style="animation-delay: 0ms"></div>
											<div class="h-2 w-2 animate-bounce rounded-full bg-indigo-400" style="animation-delay: 150ms"></div>
											<div class="h-2 w-2 animate-bounce rounded-full bg-indigo-400" style="animation-delay: 300ms"></div>
										</div>
									{:else}
										<div class="prose prose-sm prose-invert max-w-none prose-headings:text-gray-100 prose-headings:font-semibold prose-p:text-gray-200 prose-strong:text-gray-100 prose-code:text-indigo-300 prose-code:bg-gray-900/50 prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-xs prose-pre:bg-gray-900 prose-pre:border prose-pre:border-gray-700 prose-li:text-gray-300 prose-a:text-indigo-400">
											{@html renderMarkdown(msg.content)}
										</div>
										{#if msg.streaming}
											<span class="inline-block h-4 w-0.5 animate-pulse bg-indigo-400"></span>
										{/if}
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Error -->
		{#if error}
			<div class="mx-6 mb-2 rounded-lg border border-red-900/50 bg-red-950/50 px-4 py-2 text-sm text-red-400">
				{error}
				<button onclick={() => error = ''} class="ml-2 text-red-500 hover:text-red-300">dismiss</button>
			</div>
		{/if}

		<!-- Input -->
		<div class="border-t border-gray-800 bg-gray-950/50 p-4">
			<div class="mx-auto flex max-w-3xl items-end gap-3">
				<textarea
					bind:this={textareaEl}
					bind:value={inputText}
					onkeydown={handleKeydown}
					oninput={autoResize}
					placeholder="Message RUSVEL..."
					rows="1"
					disabled={sending}
					class="flex-1 resize-none rounded-xl border border-gray-700 bg-gray-800 px-4 py-3 text-sm leading-relaxed text-gray-200 placeholder-gray-500 focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500/30 disabled:opacity-50"
				></textarea>
				<button
					onclick={sendMessage}
					disabled={sending || !inputText.trim()}
					class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-xl bg-indigo-600 transition-colors hover:bg-indigo-500 disabled:opacity-30"
				>
					{#if sending}
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"></div>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
							<path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
						</svg>
					{/if}
				</button>
			</div>
		</div>
	</div>
</div>
