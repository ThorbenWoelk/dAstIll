# PRD: macOS Wake Recovery

## Problem

The local reader can fail after a Mac wakes from sleep even though the dAstIll launcher still
reports the services as running. The launcher checks shallow HTTP availability every five seconds,
but it does not react to the macOS wake event or reset stale browser, dev-server, and API
connections.

## Goal

When the local stack was meant to be running before sleep, restart it automatically after macOS
wakes so the browser can reconnect to a clean frontend and backend. Never start a stack the user
explicitly stopped.

## Current Increment

Teach the macOS launcher to remember whether the stack should be running, observe the AppKit wake
notification, and run the existing clean restart path after a short wake grace period.

## Requirements

- Adopt an already-running stack when the launcher first checks local services.
- Remember an explicit Start as intent to keep the stack running.
- Clear that intent on explicit Stop.
- Observe `NSWorkspace.didWakeNotification` through `NSWorkspace.shared.notificationCenter`.
- After wake, wait briefly for macOS networking and filesystems to settle, then restart the whole
  stack when running intent is set.
- Coalesce duplicate wake notifications and do not overlap a manual start, stop, or startup.
- Show wake recovery in the launcher status while it is happening.
- Document the behavior in the local-development guide.

## Non-Goals

- Starting the stack at login.
- Restarting a stack that was stopped before sleep.
- Polling or background recovery in the browser application.
- Changing production runtime behavior.
