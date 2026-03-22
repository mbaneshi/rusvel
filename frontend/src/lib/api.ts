// In dev mode (vite), API is on :3000. In production, same origin.
const BASE = import.meta.env.DEV ? 'http://localhost:3000' : '';

export interface SessionSummary {
	id: string;
	name: string;
	kind: string;
	tags: string[];
	updated_at: string;
}

export interface Session {
	id: string;
	name: string;
	kind: string;
	tags: string[];
	config: Record<string, unknown>;
	created_at: string;
	updated_at: string;
	metadata: Record<string, unknown>;
}

export interface Goal {
	id: string;
	session_id: string;
	title: string;
	description: string;
	timeframe: string;
	status: string;
	progress: number;
	metadata: Record<string, unknown>;
}

export interface Task {
	id: string;
	session_id: string;
	goal_id: string | null;
	title: string;
	status: string;
	due_at: string | null;
	priority: string;
	metadata: Record<string, unknown>;
}

export interface DailyPlan {
	date: string;
	tasks: Task[];
	focus_areas: string[];
	notes: string;
	metadata: Record<string, unknown>;
}

export interface Event {
	id: string;
	session_id: string | null;
	run_id: string | null;
	source: string;
	kind: string;
	payload: unknown;
	created_at: string;
	metadata: Record<string, unknown>;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...options
	});
	if (!res.ok) {
		const text = await res.text();
		throw new Error(`API error ${res.status}: ${text}`);
	}
	return res.json();
}

export async function checkHealth(): Promise<{ status: string }> {
	return request('/api/health');
}

export async function getSessions(): Promise<SessionSummary[]> {
	return request('/api/sessions');
}

export async function createSession(name: string, kind: string): Promise<{ id: string }> {
	return request('/api/sessions', {
		method: 'POST',
		body: JSON.stringify({ name, kind })
	});
}

export async function getSession(id: string): Promise<Session> {
	return request(`/api/sessions/${id}`);
}

export async function getMissionToday(sessionId: string): Promise<DailyPlan> {
	return request(`/api/sessions/${sessionId}/mission/today`);
}

export async function getGoals(sessionId: string): Promise<Goal[]> {
	return request(`/api/sessions/${sessionId}/mission/goals`);
}

export async function createGoal(
	sessionId: string,
	goal: { title: string; description: string; timeframe: string }
): Promise<Goal> {
	return request(`/api/sessions/${sessionId}/mission/goals`, {
		method: 'POST',
		body: JSON.stringify(goal)
	});
}

export async function getEvents(sessionId: string): Promise<Event[]> {
	return request(`/api/sessions/${sessionId}/events`);
}

// ── Chat (God Agent) ─────────────────────────────────────────

export interface ChatMessage {
	id: string;
	conversation_id: string;
	role: 'user' | 'assistant' | 'system';
	content: string;
	created_at: string;
}

export interface Conversation {
	id: string;
	title: string;
	updated_at: string;
	message_count: number;
}

export async function getConversations(): Promise<Conversation[]> {
	return request('/api/chat/conversations');
}

export async function getChatHistory(conversationId: string): Promise<ChatMessage[]> {
	return request(`/api/chat/conversations/${conversationId}`);
}

/**
 * Send a chat message and stream the response via SSE.
 * Calls `onDelta` for each text chunk and `onDone` when complete.
 */
export async function streamChat(
	message: string,
	conversationId: string | undefined,
	onDelta: (text: string, conversationId: string) => void,
	onDone: (fullText: string, conversationId: string) => void,
	onError: (message: string) => void
): Promise<void> {
	const res = await fetch(`${BASE}/api/chat`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ message, conversation_id: conversationId })
	});

	if (!res.ok) {
		onError(`API error ${res.status}`);
		return;
	}

	const reader = res.body?.getReader();
	if (!reader) {
		onError('No response body');
		return;
	}

	const decoder = new TextDecoder();
	let buffer = '';

	while (true) {
		const { done, value } = await reader.read();
		if (done) break;

		buffer += decoder.decode(value, { stream: true });
		const lines = buffer.split('\n');
		buffer = lines.pop() ?? '';

		for (const line of lines) {
			if (line.startsWith('event: ')) {
				// Will be followed by data line
				continue;
			}
			if (line.startsWith('data: ')) {
				const data = line.slice(6);
				try {
					const parsed = JSON.parse(data);
					if (parsed.text !== undefined && !parsed.cost_usd) {
						onDelta(parsed.text, parsed.conversation_id);
					} else if (parsed.cost_usd !== undefined) {
						onDone(parsed.text, parsed.conversation_id);
					} else if (parsed.message) {
						onError(parsed.message);
					}
				} catch {
					// Skip unparseable lines
				}
			}
		}
	}
}
