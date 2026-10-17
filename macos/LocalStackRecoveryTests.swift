@main
private enum LocalStackRecoveryTests {
  static func main() {
    // An existing local stack is adopted when the launcher opens.
    var adopted = LocalStackRecoveryIntent()
    adopted.observeInitialStack(hasServices: true)
    assert(adopted.shouldRecoverOnWake)

    // A stopped machine stays opt-in.
    var stopped = LocalStackRecoveryIntent()
    stopped.observeInitialStack(hasServices: false)
    assert(!stopped.shouldRecoverOnWake)

    // Explicit controls define wake behavior.
    stopped.startRequested()
    assert(stopped.shouldRecoverOnWake)
    stopped.stopRequested()
    assert(!stopped.shouldRecoverOnWake)

    // Later health checks must not undo an explicit choice.
    stopped.observeInitialStack(hasServices: true)
    assert(!stopped.shouldRecoverOnWake)

    print("macOS launcher recovery tests passed")
  }
}
