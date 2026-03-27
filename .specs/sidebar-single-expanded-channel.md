# Spec: Sidebar Single Expanded Channel

## Goal

Prevent the per-channel preview sidebar from presenting multiple channels as active at the same time.

## Problem

- In `per_channel_preview` mode, the channel card active styling is driven by each collection's `expanded` flag.
- Expanding another channel does not collapse previously expanded channels.
- Restored preview-session state can therefore leave multiple channels visually active at once.

## Requirements

- Expanding one preview channel must collapse every other preview channel.
- Restored preview-session state must be normalized so only one channel remains expanded.
- The behavior must be covered by automated regression tests.
