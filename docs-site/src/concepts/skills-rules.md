
## Skills

Skills are reusable prompt templates with variable placeholders. They let you save common prompts and re-invoke them with different inputs.

### Skill Structure

```
name        — Display name (e.g., "Write Blog Post")
template    — Prompt text with {variable} placeholders
department  — Which department this skill belongs to
variables   — List of variable names used in the template
```

### Example Skill

```
Name: Draft Technical Blog Post
Template: |
  Write a technical blog post about {topic}.
  Target audience: {audience}.
  Length: {length} words.
  Include code examples in {language}.
  Tone: educational but practical.
```

When invoked, you fill in `{topic}`, `{audience}`, `{length}`, and `{language}`.

### Managing Skills

#### Via the Web UI

1. Navigate to a department
2. Open the **Skills** tab in the department panel
3. Click **"Add Skill"**
4. Write the template with `{variable}` placeholders
5. Save

#### Via the API

```bash
# Create a skill
curl -X POST http://localhost:3000/api/skills \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Proposal Template",
    "template": "Write a proposal for {gig_title} on {platform}...",
    "department": "harvest"
  }'

# List all skills
curl http://localhost:3000/api/skills

# Update a skill
curl -X PUT http://localhost:3000/api/skills/<id> \
  -H "Content-Type: application/json" \
  -d '{"template": "Updated template..."}'

# Delete a skill
curl -X DELETE http://localhost:3000/api/skills/<id>
```

## Rules

Rules are constraints injected into agent system prompts. They enforce policies, coding standards, or business logic across all interactions with a department.

### Rule Structure

```
name        — Display name (e.g., "Hexagonal Architecture")
content     — The constraint text injected into system prompts
department  — Which department this rule applies to
enabled     — Whether this rule is currently active
```

### Seeded Rules

RUSVEL ships with 3 pre-configured rules:

| Rule | Department | Purpose |
|------|-----------|---------|
| **Hexagonal Architecture** | Code | Engines never import adapters. Use port traits only. |
| **Human Approval Gate** | Content, GTM | All publishing and outreach requires human approval. |
| **Crate Size Limit** | Code | Each crate must stay under 2000 lines. |

### How Rules Work

When a department's agent processes a message:

1. All **enabled** rules for that department are loaded
2. Rule content is appended to the department system prompt
3. The combined prompt guides the agent's behavior

This means rules are "always on" constraints, not one-time instructions.

### Managing Rules

#### Via the Web UI

1. Navigate to a department
2. Open the **Rules** tab
3. Toggle rules on/off with the enable switch
4. Add new rules with **"Add Rule"**

#### Via the API

```bash
# Create a rule
curl -X POST http://localhost:3000/api/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Always Write Tests",
    "content": "Every code change must include corresponding unit tests.",
    "department": "code",
    "enabled": true
  }'

# List all rules
curl http://localhost:3000/api/rules

# Toggle a rule off
curl -X PUT http://localhost:3000/api/rules/<id> \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

## Skills vs Rules

| | Skills | Rules |
|--|--------|-------|
| **Purpose** | Reusable prompts | Behavioral constraints |
| **When used** | On demand, when invoked | Always active (if enabled) |
| **Contains variables** | Yes (`{variable}`) | No |
| **Injected into** | Chat as a user message | System prompt |
