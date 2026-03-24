
## Overview

Forge is the meta-department. It orchestrates agents across all other departments and manages your strategic planning -- goals, daily plans, and periodic reviews. Think of it as the CEO's office of your virtual agency.

Forge is powered by the Forge engine, which includes the Mission sub-module for goal-driven planning.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **Daily plan** | Generate a prioritized task list for today based on active goals |
| **Review progress** | Summarize accomplishments, blockers, and next actions |
| **Set new goal** | Define a strategic objective with timeframe |

## Example Prompts

- "Generate today's mission plan based on active goals and priorities."
- "Review progress on all active goals. What is blocked?"
- "Help me define a quarterly goal for launching the product."
- "Create a workflow that runs code analysis then generates a report."
- "What should I focus on this week to hit my monthly targets?"

## CLI Commands

```bash
# Generate daily plan
cargo run -- forge mission today

# List all goals
cargo run -- forge mission goals

# Add a goal
cargo run -- forge mission goal add "Ship v1.0" --timeframe month

# Weekly review
cargo run -- forge mission review --period week

# Quarterly review
cargo run -- forge mission review --period quarter
```

## Tabs

Actions, Agents, Workflows, Skills, Rules, Events

## Related

- [First Mission guide](/getting-started/first-mission/)
- [Sessions](/concepts/sessions/) -- goals live inside sessions
- [Workflows](/concepts/workflows/) -- Forge manages cross-department workflows
