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

// Shared SSE stream parser — used by all streaming endpoints
async function parseSSE(
	res: Response,
	onChunk: (parsed: Record<string, unknown>) => void,
	onError: (message: string) => void
): Promise<void> {
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
			if (line.startsWith('event: ')) continue;
			if (line.startsWith('data: ')) {
				try {
					onChunk(JSON.parse(line.slice(6)));
				} catch {
					/* skip unparseable */
				}
			}
		}
	}
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

// ── Config (M02, M03, M04) ───────────────────────────────────

export interface ChatConfig {
	model: string;
	effort: string;
	max_budget_usd: number | null;
	permission_mode: string;
	allowed_tools: string[];
	disallowed_tools: string[];
	max_turns: number | null;
}

export interface ModelOption {
	value: string;
	label: string;
	description: string;
}

export interface ToolOption {
	name: string;
	description: string;
	category: string;
}

export async function getConfig(): Promise<ChatConfig> {
	return request('/api/config');
}

export async function updateConfig(config: ChatConfig): Promise<ChatConfig> {
	return request('/api/config', {
		method: 'PUT',
		body: JSON.stringify(config)
	});
}

export async function getModels(): Promise<ModelOption[]> {
	return request('/api/config/models');
}

export async function getTools(): Promise<ToolOption[]> {
	return request('/api/config/tools');
}

// ── Department API (shared pattern for Code/Content/Harvest/GTM) ──

export interface DepartmentConfig {
	engine: string;
	model: string;
	effort: string;
	max_budget_usd: number | null;
	permission_mode: string;
	allowed_tools: string[];
	disallowed_tools: string[];
	system_prompt: string;
	add_dirs: string[];
	max_turns: number | null;
}

export async function getDeptConfig(dept: string): Promise<DepartmentConfig> {
	return request(`/api/dept/${dept}/config`);
}

export async function updateDeptConfig(
	dept: string,
	config: DepartmentConfig
): Promise<DepartmentConfig> {
	return request(`/api/dept/${dept}/config`, { method: 'PUT', body: JSON.stringify(config) });
}

export async function getDeptConversations(dept: string): Promise<Conversation[]> {
	return request(`/api/dept/${dept}/chat/conversations`);
}

export async function getDeptChatHistory(dept: string, id: string): Promise<ChatMessage[]> {
	return request(`/api/dept/${dept}/chat/conversations/${id}`);
}

export async function getDeptEvents(dept: string): Promise<Event[]> {
	return request(`/api/dept/${dept}/events`);
}

export async function streamDeptChat(
	dept: string,
	message: string,
	conversationId: string | undefined,
	onDelta: (text: string, conversationId: string) => void,
	onDone: (fullText: string, conversationId: string) => void,
	onError: (message: string) => void
): Promise<void> {
	const res = await fetch(`${BASE}/api/dept/${dept}/chat`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ message, conversation_id: conversationId })
	});
	await parseSSE(
		res,
		(p) => {
			if (p.text !== undefined && p.cost_usd === undefined)
				onDelta(p.text as string, p.conversation_id as string);
			else if (p.cost_usd !== undefined) onDone(p.text as string, p.conversation_id as string);
			else if (p.message) onError(p.message as string);
		},
		onError
	);
}

// ── Agents CRUD ──────────────────────────────────────────────

export interface Agent {
	id: string;
	name: string;
	role: string;
	instructions: string;
	default_model: { provider: string; model: string };
	allowed_tools: string[];
	capabilities: string[];
	budget_limit: number | null;
	metadata: Record<string, unknown>;
}

export async function getAgents(engine?: string): Promise<Agent[]> {
	return request(`/api/agents${engine ? `?engine=${engine}` : ''}`);
}

export async function createAgent(agent: {
	name: string;
	role?: string;
	instructions?: string;
	model?: string;
	allowed_tools?: string[];
	budget_limit?: number;
	metadata?: Record<string, unknown>;
}): Promise<Agent> {
	return request('/api/agents', { method: 'POST', body: JSON.stringify(agent) });
}

export async function deleteAgent(id: string): Promise<void> {
	await fetch(`${BASE}/api/agents/${id}`, { method: 'DELETE' });
}

// ── Skills CRUD ──────────────────────────────────────────────

export interface Skill {
	id: string;
	name: string;
	description: string;
	prompt_template: string;
	metadata: Record<string, unknown>;
}

export async function getSkills(engine?: string): Promise<Skill[]> {
	return request(`/api/skills${engine ? `?engine=${engine}` : ''}`);
}

export async function createSkill(skill: Partial<Skill>): Promise<Skill> {
	return request('/api/skills', { method: 'POST', body: JSON.stringify(skill) });
}

export async function deleteSkill(id: string): Promise<void> {
	await fetch(`${BASE}/api/skills/${id}`, { method: 'DELETE' });
}

// ── Rules CRUD ───────────────────────────────────────────────

export interface Rule {
	id: string;
	name: string;
	content: string;
	enabled: boolean;
	metadata: Record<string, unknown>;
}

export async function getRules(engine?: string): Promise<Rule[]> {
	return request(`/api/rules${engine ? `?engine=${engine}` : ''}`);
}

export async function createRule(rule: Partial<Rule>): Promise<Rule> {
	return request('/api/rules', { method: 'POST', body: JSON.stringify(rule) });
}

export async function updateRule(id: string, rule: Partial<Rule>): Promise<Rule> {
	return request(`/api/rules/${id}`, { method: 'PUT', body: JSON.stringify(rule) });
}

export async function deleteRule(id: string): Promise<void> {
	await fetch(`${BASE}/api/rules/${id}`, { method: 'DELETE' });
}

// ── MCP Servers CRUD ─────────────────────────────────────────

export interface McpServer {
	id: string;
	name: string;
	description: string;
	server_type: string;
	command: string | null;
	args: string[];
	url: string | null;
	env: Record<string, unknown>;
	enabled: boolean;
	metadata: Record<string, unknown>;
}

export async function getMcpServers(engine?: string): Promise<McpServer[]> {
	return request(`/api/mcp-servers${engine ? `?engine=${engine}` : ''}`);
}

export async function createMcpServer(server: Partial<McpServer>): Promise<McpServer> {
	return request('/api/mcp-servers', { method: 'POST', body: JSON.stringify(server) });
}

export async function deleteMcpServer(id: string): Promise<void> {
	await fetch(`${BASE}/api/mcp-servers/${id}`, { method: 'DELETE' });
}

// ── Hooks CRUD ───────────────────────────────────────────────

export interface Hook {
	id: string;
	name: string;
	event: string;
	matcher: string;
	hook_type: string;
	action: string;
	enabled: boolean;
	metadata: Record<string, unknown>;
}

export async function getHooks(engine?: string): Promise<Hook[]> {
	return request(`/api/hooks${engine ? `?engine=${engine}` : ''}`);
}

export async function createHook(hook: Partial<Hook>): Promise<Hook> {
	return request('/api/hooks', { method: 'POST', body: JSON.stringify(hook) });
}

export async function updateHook(id: string, hook: Partial<Hook>): Promise<Hook> {
	return request(`/api/hooks/${id}`, { method: 'PUT', body: JSON.stringify(hook) });
}

export async function deleteHook(id: string): Promise<void> {
	await fetch(`${BASE}/api/hooks/${id}`, { method: 'DELETE' });
}

export async function getHookEvents(): Promise<string[]> {
	return request('/api/hooks/events');
}

// ── Workflows CRUD + Execution ────────────────────────────────

export interface WorkflowStepDef {
	agent_name: string;
	prompt_template: string;
	step_type: string;
}

export interface Workflow {
	id: string;
	name: string;
	description: string;
	steps: WorkflowStepDef[];
	metadata: Record<string, unknown>;
}

export interface StepResult {
	step_index: number;
	agent_name: string;
	prompt: string;
	output: string;
	cost_usd: number;
}

export interface WorkflowRunResult {
	workflow_id: string;
	workflow_name: string;
	steps: StepResult[];
	total_cost_usd: number;
}

export async function getWorkflows(): Promise<Workflow[]> {
	return request('/api/workflows');
}

export async function createWorkflow(workflow: {
	name: string;
	description?: string;
	steps: WorkflowStepDef[];
	metadata?: Record<string, unknown>;
}): Promise<Workflow> {
	return request('/api/workflows', { method: 'POST', body: JSON.stringify(workflow) });
}

export async function updateWorkflow(id: string, workflow: Workflow): Promise<Workflow> {
	return request(`/api/workflows/${id}`, { method: 'PUT', body: JSON.stringify(workflow) });
}

export async function deleteWorkflow(id: string): Promise<void> {
	await fetch(`${BASE}/api/workflows/${id}`, { method: 'DELETE' });
}

export async function runWorkflow(
	id: string,
	variables?: Record<string, string>
): Promise<WorkflowRunResult> {
	return request(`/api/workflows/${id}/run`, {
		method: 'POST',
		body: JSON.stringify({ variables: variables ?? {} })
	});
}

// ── Approvals (Human-in-the-loop, ADR-008) ───────────────────

export interface Job {
	id: string;
	session_id: string;
	kind: string;
	payload: unknown;
	status: string;
	scheduled_at: string | null;
	started_at: string | null;
	completed_at: string | null;
	retries: number;
	max_retries: number;
	error: string | null;
	metadata: Record<string, unknown>;
}

export async function getPendingApprovals(): Promise<Job[]> {
	return request('/api/approvals');
}

export async function approveJob(id: string): Promise<void> {
	return request(`/api/approvals/${id}/approve`, { method: 'POST' });
}

export async function rejectJob(id: string): Promise<void> {
	return request(`/api/approvals/${id}/reject`, { method: 'POST' });
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

// ── Capability Engine ────────────────────────────────────────

export async function streamCapability(
	description: string,
	engine: string | undefined,
	onDelta: (text: string) => void,
	onDone: (fullText: string) => void,
	onError: (message: string) => void
): Promise<void> {
	const res = await fetch(`${BASE}/api/capability/build`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ description, engine })
	});
	await parseSSE(
		res,
		(p) => {
			if (p.text !== undefined && p.cost_usd === undefined) onDelta(p.text as string);
			else if (p.cost_usd !== undefined) onDone(p.text as string);
			else if (p.message) onError(p.message as string);
		},
		onError
	);
}

// ── Help (AI-powered) ────────────────────────────────────────

export async function streamHelp(
	question: string,
	context: string | undefined,
	onDelta: (text: string) => void,
	onDone: (fullText: string) => void,
	onError: (message: string) => void
): Promise<void> {
	const res = await fetch(`${BASE}/api/help`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ question, context })
	});
	await parseSSE(
		res,
		(p) => {
			if (p.text !== undefined && p.cost_usd === undefined) onDelta(p.text as string);
			else if (p.cost_usd !== undefined) onDone(p.text as string);
			else if (p.message) onError(p.message as string);
		},
		onError
	);
}

// ── Chat (God Agent) ─────────────────────────────────────────

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
	await parseSSE(
		res,
		(p) => {
			if (p.text !== undefined && p.cost_usd === undefined)
				onDelta(p.text as string, p.conversation_id as string);
			else if (p.cost_usd !== undefined) onDone(p.text as string, p.conversation_id as string);
			else if (p.message) onError(p.message as string);
		},
		onError
	);
}

// ── Department Registry ──────────────────────────────────────

export interface QuickAction {
	label: string;
	prompt: string;
}

export interface DepartmentDef {
	id: string;
	name: string;
	title: string;
	engine_kind: string;
	icon: string;
	color: string;
	system_prompt: string;
	capabilities: string[];
	tabs: string[];
	quick_actions: QuickAction[];
	default_config: Record<string, unknown>;
}

export async function getDepartments(): Promise<DepartmentDef[]> {
	const res = await fetch(`${BASE}/api/departments`);
	return res.json();
}

export async function getProfile(): Promise<unknown> {
	const res = await fetch(`${BASE}/api/profile`);
	return res.json();
}

export async function updateProfile(profile: unknown): Promise<unknown> {
	const res = await fetch(`${BASE}/api/profile`, {
		method: 'PUT',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(profile)
	});
	return res.json();
}

// ── Knowledge (RAG) ───────────────────────────────────────────

export interface KnowledgeEntry {
	id: string;
	content: string;
	source: string;
	created_at: string;
	metadata: Record<string, unknown>;
}

export interface KnowledgeSearchResult {
	entry: KnowledgeEntry;
	score: number;
}

export interface KnowledgeStats {
	total_entries: number;
	model_name: string;
	dimensions: number;
}

export async function ingestKnowledge(
	content: string,
	source: string
): Promise<{ chunks_stored: number }> {
	return request('/api/knowledge/ingest', {
		method: 'POST',
		body: JSON.stringify({ content, source })
	});
}

export async function getKnowledge(): Promise<KnowledgeEntry[]> {
	return request('/api/knowledge');
}

export async function deleteKnowledge(id: string): Promise<void> {
	await fetch(`${BASE}/api/knowledge/${id}`, { method: 'DELETE' });
}

export async function searchKnowledge(
	query: string,
	limit?: number
): Promise<KnowledgeSearchResult[]> {
	return request('/api/knowledge/search', {
		method: 'POST',
		body: JSON.stringify({ query, limit: limit ?? 5 })
	});
}

export async function getKnowledgeStats(): Promise<KnowledgeStats> {
	return request('/api/knowledge/stats');
}

// ── Visual Testing ────────────────────────────────────────────

export interface VisualIssue {
	type: string;
	description: string;
	element: string;
	suggested_fix: string;
}

export interface RouteAnalysis {
	route: string;
	severity: 'low' | 'medium' | 'high' | 'critical';
	issues: VisualIssue[];
	recommended_actions: { action_type: string; entity_description: string }[];
}

export interface VisualReport {
	run_id: string;
	timestamp: string;
	analyses: RouteAnalysis[];
	summary: {
		total_routes: number;
		regressions: number;
		critical: number;
		high: number;
		medium: number;
		low: number;
	};
}

export interface CorrectionResult {
	skills_created: string[];
	rules_created: string[];
	errors: string[];
}

export async function getVisualReports(): Promise<VisualReport[]> {
	return request('/api/system/visual-report');
}

export async function postVisualReport(report: VisualReport): Promise<{ id: string }> {
	return request('/api/system/visual-report', {
		method: 'POST',
		body: JSON.stringify(report)
	});
}

export async function triggerSelfCorrect(): Promise<CorrectionResult> {
	return request('/api/system/visual-report/self-correct', { method: 'POST' });
}

export async function runVisualTests(): Promise<{
	success: boolean;
	stdout: string;
	stderr: string;
}> {
	return request('/api/system/visual-test', { method: 'POST' });
}

// ── Analytics ─────────────────────────────────────────────────
export interface AnalyticsData {
	agents: number;
	skills: number;
	rules: number;
	mcp_servers: number;
	hooks: number;
	conversations: number;
	events: number;
	departments: number;
}

export async function getAnalytics(): Promise<AnalyticsData> {
	const res = await fetch(`${BASE}/api/analytics`);
	if (!res.ok) throw new Error('Failed to load analytics');
	return res.json();
}
