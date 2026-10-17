struct LocalStackRecoveryIntent {
  private(set) var shouldRecoverOnWake = false
  private var observedInitialStack = false

  mutating func observeInitialStack(hasServices: Bool) {
    guard !observedInitialStack else { return }
    observedInitialStack = true
    shouldRecoverOnWake = hasServices
  }

  mutating func startRequested() {
    observedInitialStack = true
    shouldRecoverOnWake = true
  }

  mutating func stopRequested() {
    observedInitialStack = true
    shouldRecoverOnWake = false
  }
}
