---
title: Content Department
description: Content creation, platform adaptation, and publishing strategy.
---

## Overview

The Content department handles all content creation -- blog posts, social media threads, documentation, email newsletters, and video scripts. It drafts in Markdown and adapts content for multiple platforms. All publishing goes through a human approval gate before going live.

Powered by the Content engine with its write > adapt > review > approve > publish pipeline.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **Draft blog post** | Start a new long-form blog post |
| **Adapt for Twitter** | Convert content into a Twitter/X thread |
| **Content calendar** | View scheduled and draft content for the week |

## Example Prompts

- "Draft a blog post about hexagonal architecture in Rust."
- "Adapt the latest blog post into a LinkedIn post."
- "Create a content calendar for the next two weeks."
- "Write a technical tutorial on setting up SQLite WAL mode."
- "Review my draft and suggest improvements for engagement."

## Content Types

The department handles these content kinds: LongForm, Tweet, Thread, LinkedInPost, Blog, VideoScript, Email, and Proposal.

## Approval Gate

Content publishing always requires human approval by default (ADR-008). When content is ready to publish, it enters the `AwaitingApproval` state. You review and approve or reject from the UI or API.

## Tabs

Actions, Agents, Skills, Rules, Events

## Seeded Agent

| Agent | Role |
|-------|------|
| **content-writer** | Drafts blog posts, social content, and documentation |
