---
title: GTM Department
description: CRM, outreach sequences, deal management, and invoicing.
---

## Overview

The GoToMarket (GTM) department handles everything about running your business relationships -- CRM contacts, outreach email sequences, deal pipeline tracking, and invoicing. It is the merged result of the original Ops and Connect engines (ADR-001).

All outreach sending goes through a human approval gate before delivery.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **List contacts** | Show all CRM contacts with status and last interaction |
| **Draft outreach** | Create a multi-step email outreach sequence |
| **Deal pipeline** | View deals by stage, value, and next actions |
| **Generate invoice** | Create an invoice for a client |

## Example Prompts

- "List all contacts in the CRM with their company and last interaction date."
- "Draft a 3-email outreach sequence for cold leads in the SaaS space."
- "Show the current deal pipeline sorted by expected close date."
- "Generate an invoice for Acme Corp: 40 hours at $150/hr."
- "What deals are stale and need follow-up?"

## Deal Stages

Deals flow through: **Cold** > **Contacted** > **Qualified** > **ProposalSent** > **Won** / **Lost**

## Approval Gate

Outreach sending always requires human approval (ADR-008). When a sequence is ready to send, you review the email content and recipient list before confirming.

## Tabs

Actions, Agents, Workflows, Skills, Rules, Events
