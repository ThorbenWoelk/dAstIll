import AppKit
import SwiftUI

private enum Service: String, CaseIterable, Identifiable {
  case reader = "Reader"
  case backend = "Backend"
  case docs = "Docs"
  case transcription = "Transcription"

  var id: Self { self }

  var url: URL {
    switch self {
    case .reader: URL(string: "http://127.0.0.1:3543/")!
    case .backend: URL(string: "http://127.0.0.1:3544/api/health")!
    case .docs: URL(string: "http://127.0.0.1:4173/")!
    case .transcription: URL(string: "http://127.0.0.1:5092/health")!
    }
  }
}

@MainActor
private final class LocalStack: ObservableObject {
  @Published private(set) var available: Set<Service> = []
  @Published private(set) var hasChecked = false
  @Published private(set) var isWorking = false
  @Published private(set) var isStarting = false
  @Published private(set) var message: String?

  let repository: URL
  private let requiredServices: Set<Service>
  private var timer: Timer?
  private var startupMonitor: Task<Void, Never>?

  init() {
    let path = Bundle.main.object(forInfoDictionaryKey: "DastillRepositoryPath") as? String ?? ""
    repository = URL(fileURLWithPath: path, isDirectory: true)
    var required: Set<Service> = [.reader, .backend, .docs]
    let backendEnvironment = repository.appendingPathComponent("backend/.env")
    if let content = try? String(contentsOf: backendEnvironment, encoding: .utf8),
      content.split(separator: "\n").contains(where: {
        let line = $0.trimmingCharacters(in: .whitespaces)
        return line == "LOCAL_ASR_ENABLED=true" || line == "LOCAL_ASR_ENABLED=1"
      })
    {
      required.insert(.transcription)
    }
    requiredServices = required
    Task { await refresh() }
    timer = Timer.scheduledTimer(withTimeInterval: 5, repeats: true) { [weak self] _ in
      Task { @MainActor in await self?.refresh() }
    }
  }

  var isRunning: Bool {
    available.isSuperset(of: requiredServices)
  }

  var status: String {
    if !hasChecked { return "Checking local services…" }
    if isWorking { return "Changing local services…" }
    if isRunning { return "Running on this Mac" }
    if isStarting { return "Starting on this Mac…" }
    if available.isEmpty { return "Stopped" }
    return "Some services are unavailable"
  }

  var statusSymbol: String {
    if !hasChecked { return "arrow.triangle.2.circlepath" }
    if isWorking || isStarting { return "arrow.triangle.2.circlepath" }
    if isRunning { return "checkmark.circle.fill" }
    if available.isEmpty { return "stop.circle" }
    return "exclamationmark.triangle.fill"
  }

  func refresh() async {
    var found = Set<Service>()
    await withTaskGroup(of: (Service, Bool).self) { group in
      for service in Service.allCases {
        group.addTask { (service, await Self.isHealthy(service.url)) }
      }
      for await (service, healthy) in group where healthy {
        found.insert(service)
      }
    }
    available = found
    hasChecked = true
    if isRunning && isStarting {
      isStarting = false
      message = nil
      startupMonitor?.cancel()
      startupMonitor = nil
    }
  }

  func start() {
    guard hasChecked && !isWorking && !isStarting else { return }
    isWorking = true
    message = nil
    let repositoryPath = repository.path
    let hadServices = !available.isEmpty
    Task {
      do {
        if hadServices {
          try await Task.detached(priority: .userInitiated) {
            try Self.runScript("end_app.sh", arguments: [], in: repositoryPath)
          }.value
          await refresh()
        }
        try await Task.detached(priority: .userInitiated) {
          try Self.runScript("start_app.sh", arguments: ["--detach"], in: repositoryPath)
        }.value
        isStarting = true
        isWorking = false
        startupMonitor = Task {
          for _ in 0..<150 {
            await refresh()
            if isRunning || Task.isCancelled { return }
            try? await Task.sleep(for: .seconds(2))
          }
          isStarting = false
          message = "Startup is still incomplete. Open the startup log for details."
        }
        return
      } catch {
        message = "Could not start dAstIll: \(error.localizedDescription)"
      }
      isWorking = false
      await refresh()
    }
  }

  func stop() {
    guard hasChecked && !isWorking else { return }
    startupMonitor?.cancel()
    startupMonitor = nil
    isStarting = false
    isWorking = true
    message = nil
    let repositoryPath = repository.path
    Task {
      do {
        try await Task.detached(priority: .userInitiated) {
          try Self.runScript("end_app.sh", arguments: [], in: repositoryPath)
        }.value
      } catch {
        message = "Could not stop dAstIll: \(error.localizedDescription)"
      }
      isWorking = false
      await refresh()
    }
  }

  func openReader() {
    NSWorkspace.shared.open(URL(string: "http://localhost:3543/")!)
  }

  func openLog() {
    NSWorkspace.shared.open(repository.appendingPathComponent("start_app.log"))
  }

  private nonisolated static func isHealthy(_ url: URL) async -> Bool {
    var request = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData)
    request.timeoutInterval = 2
    do {
      let (_, response) = try await URLSession.shared.data(for: request)
      return (response as? HTTPURLResponse)?.statusCode == 200
    } catch {
      return false
    }
  }

  private nonisolated static func runScript(
    _ name: String,
    arguments: [String],
    in repositoryPath: String
  ) throws {
    let script = URL(fileURLWithPath: repositoryPath).appendingPathComponent(name)
    guard FileManager.default.isExecutableFile(atPath: script.path) else {
      throw LauncherError.missingScript(script.path)
    }

    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/bin/zsh")
    process.arguments = [script.path] + arguments
    process.currentDirectoryURL = URL(fileURLWithPath: repositoryPath, isDirectory: true)
    var environment = ProcessInfo.processInfo.environment
    let home = environment["HOME"] ?? NSHomeDirectory()
    environment["PATH"] = [
      "/opt/homebrew/bin", "/usr/local/bin", "\(home)/.cargo/bin", "\(home)/.bun/bin",
      environment["PATH"] ?? "/usr/bin:/bin:/usr/sbin:/sbin",
    ].joined(separator: ":")
    process.environment = environment
    let output = Pipe()
    process.standardOutput = output
    process.standardError = output
    try process.run()
    let data = output.fileHandleForReading.readDataToEndOfFile()
    process.waitUntilExit()
    if process.terminationStatus != 0 {
      let detail = String(decoding: data, as: UTF8.self)
        .trimmingCharacters(in: .whitespacesAndNewlines)
      throw LauncherError.scriptFailed(
        detail.isEmpty ? "Exit \(process.terminationStatus)" : detail)
    }
  }
}

private enum LauncherError: LocalizedError {
  case missingScript(String)
  case scriptFailed(String)

  var errorDescription: String? {
    switch self {
    case .missingScript(let path): "Missing script at \(path). Reinstall the launcher."
    case .scriptFailed(let detail): detail
    }
  }
}

private struct LauncherWindow: View {
  @ObservedObject var stack: LocalStack

  var body: some View {
    VStack(alignment: .leading, spacing: 18) {
      HStack(spacing: 10) {
        Image(systemName: stack.statusSymbol)
          .foregroundStyle(stack.isRunning ? .green : .secondary)
        Text(stack.status).font(.headline)
      }

      VStack(alignment: .leading, spacing: 8) {
        ForEach(Service.allCases) { service in
          HStack {
            Text(service.rawValue)
            Spacer()
            Text(stack.available.contains(service) ? "Running" : "Stopped")
              .foregroundStyle(.secondary)
          }
        }
      }

      if let message = stack.message {
        Text(message).foregroundStyle(.red).textSelection(.enabled)
      }

      HStack {
        Button(stack.isRunning ? "Restart" : "Start") { stack.start() }
          .disabled(!stack.hasChecked || stack.isWorking || stack.isStarting)
        Button("Stop") { stack.stop() }
          .disabled(
            !stack.hasChecked || stack.isWorking || (stack.available.isEmpty && !stack.isStarting))
        Spacer()
        Button("Open App") { stack.openReader() }
          .disabled(!stack.available.contains(.reader))
        Button("Open Log") { stack.openLog() }
      }
    }
    .padding(24)
    .frame(minWidth: 420)
    .task { await stack.refresh() }
  }
}

private struct LauncherMenu: View {
  @ObservedObject var stack: LocalStack
  @Environment(\.openWindow) private var openWindow

  var body: some View {
    Text("dAstIll Local")
    Text(stack.status)
    Divider()
    Button("Show Controls") {
      openWindow(id: "controls")
      NSApp.activate(ignoringOtherApps: true)
    }
    Button(stack.isRunning ? "Restart" : "Start") { stack.start() }
      .disabled(!stack.hasChecked || stack.isWorking || stack.isStarting)
    Button("Stop") { stack.stop() }
      .disabled(
        !stack.hasChecked || stack.isWorking || (stack.available.isEmpty && !stack.isStarting))
    Button("Open App") { stack.openReader() }
      .disabled(!stack.available.contains(.reader))
    Button("Open Log") { stack.openLog() }
    Divider()
    Button("Quit Launcher (services stay running)") { NSApp.terminate(nil) }
  }
}

@main
private struct DastillLauncherApp: App {
  @StateObject private var stack = LocalStack()
  private let menuBarIcon: NSImage = {
    guard let url = Bundle.main.url(forResource: "dastill", withExtension: "icns"),
      let image = NSImage(contentsOf: url)
    else {
      return NSImage()
    }
    return image
  }()

  var body: some Scene {
    WindowGroup("dAstIll Local", id: "controls") {
      LauncherWindow(stack: stack)
    }
    .windowResizability(.contentSize)

    MenuBarExtra {
      LauncherMenu(stack: stack)
    } label: {
      Image(nsImage: menuBarIcon)
        .resizable()
        .interpolation(.high)
        .frame(width: 18, height: 18)
        .opacity(stack.isRunning ? 1 : 0.5)
        .accessibilityLabel("dAstIll Local — \(stack.status)")
        .help("dAstIll Local — \(stack.status)")
    }
  }
}
