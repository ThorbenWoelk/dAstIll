# Test Spec: macOS Wake Recovery

## Acceptance Criteria

- [ ] An already-running stack is marked for wake recovery after the initial service check.
- [ ] Pressing Start marks the stack for wake recovery.
- [ ] Pressing Stop disables wake recovery.
- [ ] A wake event restarts the full stack after a short grace period when recovery is enabled.
- [ ] A wake event leaves a deliberately stopped stack stopped.
- [ ] Repeated wake events cannot create overlapping restarts.
- [ ] The launcher compiles for its macOS 13 deployment target.

## Automated Checks

`scripts/test_macos_launcher.sh` compiles and runs a small Swift test executable covering the pure
recovery-intent state transitions:

- initial running-stack adoption;
- initial stopped state;
- Start followed by wake eligibility;
- Stop followed by wake ineligibility;
- repeated initial observations do not overwrite an explicit choice.

The installer compile command is also run against the launcher sources to catch AppKit and SwiftUI
integration errors.

## Manual Check

1. Start dAstIll from the launcher and leave the reader open in a browser.
2. Put the Mac to sleep, then wake it.
3. Confirm the launcher briefly reports wake recovery and then returns to “Running on this Mac”.
4. Confirm the existing browser tab reconnects and renders the reader without a manual restart.
5. Stop dAstIll, sleep and wake again, and confirm the stack stays stopped.
