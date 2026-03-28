
## Overview

The Messaging department handles outbound notification channels. It provides a unified interface for sending messages across platforms like Telegram, with support for additional channels as they are wired.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **Send notification** | Send a message via a configured channel |
| **Channel status** | Check which notification channels are active |

## Example Prompts

- "Send a Telegram notification about the latest deployment."
- "What notification channels are currently configured?"
- "Notify me when the harvest pipeline finds a new opportunity."

## Channels

- **Telegram** -- enabled when `RUSVEL_TELEGRAM_BOT_TOKEN` is set; sends via the `POST /api/system/notify` endpoint

## Notes

The Messaging department is backed by `dept-messaging` (a `DepartmentApp` shell); it does not yet have a separate engine crate. Outbound delivery is provided by `rusvel-channel` and the `ChannelPort` trait.

## Tabs

Actions, Agents, Rules, Events
