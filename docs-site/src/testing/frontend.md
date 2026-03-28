# Frontend Testing

Start the server (`cargo run`) and open `http://localhost:3000` in a browser.

## Pages to Check

| URL | What to verify |
|-----|---------------|
| `/` | Dashboard loads. Shows department cards, system status, recent activity |
| `/chat` | Chat interface loads. Can type messages, SSE stream works |
| `/dept/forge` | Forge overview. Shows mission, goals, tasks |
| `/dept/code` | Code department. Shows analysis tools |
| `/dept/content` | Content department. Shows drafting UI |
| `/dept/harvest` | Harvest department. Shows pipeline |
| `/dept/gtm` | GTM department. Shows CRM |
| `/dept/finance` | Finance department. Shows ledger |
| `/dept/[id]/chat` | Department-scoped chat works for each department |
| `/dept/[id]/agents` | Agent CRUD UI -- list, create, edit, delete |
| `/dept/[id]/skills` | Skills CRUD UI |
| `/dept/[id]/rules` | Rules CRUD UI |
| `/dept/[id]/hooks` | Hooks CRUD UI |
| `/dept/[id]/mcp` | MCP server management |
| `/dept/[id]/workflows` | Workflow management |
| `/dept/[id]/config` | Department configuration |
| `/dept/[id]/events` | Event log |
| `/dept/[id]/actions` | Quick actions |
| `/dept/[id]/engine` | Engine-specific UI |
| `/dept/[id]/terminal` | Terminal pane |
| `/dept/content/calendar` | Content calendar view |
| `/dept/harvest/pipeline` | Opportunity pipeline view |
| `/dept/gtm/contacts` | CRM contacts |
| `/dept/gtm/deals` | CRM deals |
| `/dept/gtm/outreach` | Outreach sequences |
| `/dept/gtm/invoices` | Invoicing |
| `/flows` | Flow builder -- create/edit DAG workflows |
| `/knowledge` | Knowledge base -- ingest, search |
| `/approvals` | Approval queue -- pending human approvals (sidebar badge) |
| `/settings` | Global settings |
| `/settings/spend` | Cost analytics |
| `/terminal` | Terminal view |
| `/database/tables` | Database browser -- list tables |
| `/database/schema` | Schema viewer |
| `/database/sql` | SQL runner |

## Feature Checklist

| Feature | How to test | Expected |
|---------|------------|----------|
| Navigation | Click each sidebar item | Page loads without error |
| Department switching | Click different departments | URL updates, content refreshes |
| Dark mode | Toggle theme (if available) | CSS variables swap, no broken colors |
| Chat streaming | Send a message in `/chat` | Text appears character by character |
| Tool call cards | Chat with code dept, ask about files | ToolCallCard renders with tool name + result |
| Approval cards | Create content draft needing approval | ApprovalCard appears in sidebar badge |
| Department colors | Visit each department | Each has its own oklch color accent |
| Responsive layout | Resize browser window | Layout adapts, no overflow |
| Error handling | Visit `/dept/nonexistent` | Error page or redirect, no crash |
| Command palette | Keyboard shortcut (Cmd+K or similar) | CommandPalette opens |
| Onboarding | Fresh install, first visit | OnboardingChecklist or ProductTour appears |

## Visual Regression Tests

```bash
cd frontend
pnpm install
pnpm test:visual
```

**Expected:** Playwright takes screenshots and compares against baselines.

```bash
# Update baselines
pnpm test:e2e:update
```

```bash
# AI-powered diff analysis
pnpm test:analyze
```

**Expected:** Claude Vision analyzes visual diffs and reports issues.

```bash
# Via API
curl -s -X POST http://localhost:3000/api/system/visual-test | jq .

# Self-correction loop
curl -s -X POST http://localhost:3000/api/system/visual-report/self-correct | jq .
```

**Expected:** Auto-generates fix skills/rules based on visual diff analysis.
