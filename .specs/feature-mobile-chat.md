# Feature: Mobile Chat

Priority: **P1**

## Outcome

Make chat feel intentionally mobile rather than a compressed desktop layout.

## Source Of Truth

Use [design.md](../design.md) for shell, touch, and testing expectations. Keep chat-specific behavior here.

## Consolidated Scope

This feature replaces the intent of:

- `08-mobile-chat.md`

It should also coordinate with:

- [feature-mobile-shell-navigation](./feature-mobile-shell-navigation.md)
- [feature-mobile-interaction-accessibility](./feature-mobile-interaction-accessibility.md)

## Requirements

### 1. Compact Input Footprint

The chat composer should preserve message viewport space on small screens.

Acceptance:

- controls do not expand into a needlessly tall stack
- the textarea remains usable
- source/deep-research/send affordances remain understandable on touch

### 2. Clear Conversation Access

Mobile users must have an obvious way to open and manage conversations.

Acceptance:

- conversation access is visible in the mobile top bar or an equivalent first-class entry point
- rename/delete flows inside the mobile conversation surface work without hover

### 3. Better Message-Space Utilization

Chat content should use mobile width efficiently.

Acceptance:

- excessive horizontal padding is reduced where it currently wastes space
- message content remains readable
- jump-to-latest and similar controls meet touch expectations

## Acceptance Criteria

- a mobile user can open the conversation list, switch conversations, and manage them without hidden hover interactions
- the chat input stays compact enough that message reading remains comfortable on a phone-sized viewport
- visible chat interaction changes have Playwright coverage

## Verification

Required:

- `bun install --frozen-lockfile`
- `bun run format:check`
- `bun run lint`
- `bun run check`
- `bun run test`
- `bun run build`
- `bun audit --production`

Required Playwright coverage:

- mobile conversation access path
- one mobile chat-input/layout assertion
- one mobile message/action assertion if related DOM changes land
