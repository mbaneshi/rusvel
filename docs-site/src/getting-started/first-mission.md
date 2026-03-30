
## What is a Mission?

A mission is RUSVEL's daily planning system, powered by the Forge engine. It reads your active goals, checks the state of all departments, and generates a prioritized task list for the day.

The workflow is: **set goals** > **generate daily plan** > **execute** > **review progress**.

## Set Goals

Goals are high-level objectives with a timeframe. They drive what the daily plan prioritizes.

### Via the Web UI

1. Navigate to the **Forge** department from the sidebar
2. Click the **"Set new goal"** quick action
3. The Forge agent will ask for context and help you define the goal

### Via the CLI

```bash
# Add a goal with default month timeframe
cargo run -- forge mission goal add "Launch MVP"

# Add a goal with specific timeframe and description
cargo run -- forge mission goal add "Get 10 beta users" \
  --description "Recruit beta testers from HN and Reddit" \
  --timeframe week

# List all goals
cargo run -- forge mission goals
```

Output from `goals`:

```
ID                                      TITLE                      TIMEFRAME   STATUS      PROGRESS
----------------------------------------------------------------------------------------------------
a1b2c3d4-...                            Launch MVP                 Month       Active      0%
e5f6g7h8-...                            Get 10 beta users          Week        Active      0%
```

### Goal Timeframes

| Timeframe | Planning Horizon |
|-----------|-----------------|
| `day` | Today only |
| `week` | This week |
| `month` | This month |
| `quarter` | This quarter |

## Generate a Daily Plan

The daily plan is an AI-generated prioritized task list based on your goals and current state.

### Via the Web UI

1. Open the **Forge** department
2. Click **"Daily plan"** quick action
3. The agent generates tasks with priorities and focus areas

### Via the CLI

```bash
cargo run -- forge mission today
```

Output:

```
Generating daily plan...

Daily Plan -- 2026-03-23
==================================================
  1. [High] Finalize API endpoint documentation
  2. [High] Write integration tests for auth flow
  3. [Medium] Draft landing page copy
  4. [Medium] Set up CI pipeline
  5. [Low] Review dependency updates

Focus areas:
  - Ship the authentication feature
  - Prepare for beta launch

Notes: Focus on high-priority items first. The auth flow is blocking beta signups.
```

## Review Progress

Periodic reviews summarize what was accomplished and identify blockers.

### Via the CLI

```bash
# Weekly review (default)
cargo run -- forge mission review

# Monthly review
cargo run -- forge mission review --period month

# Quarterly review
cargo run -- forge mission review --period quarter
```

Output:

```
Generating Week review...

Review (Week)
==================================================

Accomplishments:
  - Completed API authentication flow
  - Deployed staging environment
  - Drafted 3 blog posts

Blockers:
  - Waiting on SSL certificate for production
  - Need feedback on pricing page design

Insights:
  - Code velocity increased 40% after adding test automation
  - Content pipeline is 2 days behind schedule

Next actions:
  - Follow up on SSL certificate
  - Schedule design review for pricing page
```

## Explore Other Departments

With your mission set, explore the **14** department apps. Each has its own AI agent specialized for that domain:

- **[Code](/departments/code/)** -- implement features, analyze code, run tests
- **[Content](/departments/content/)** -- write blog posts, adapt for social platforms
- **[Harvest](/departments/harvest/)** -- find freelance opportunities, draft proposals
- **[GTM](/departments/gtm/)** -- manage CRM, send outreach, track deals

See the full list in [Departments](/concepts/departments/).
