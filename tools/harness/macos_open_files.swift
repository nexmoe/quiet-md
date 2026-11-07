// swift tools/harness/macos_open_files.swift ~/Applications/Quiet-md-Test.app
// Use a devtools build in a disposable bundle, never the installed app.
import AppKit
import CoreServices

var runningTest: NSRunningApplication?
func require(_ ok: Bool, _ message: String) {
    if !ok {
        fputs("FAIL: \(message)\n", stderr)
        if let instance = runningTest {
            instance.terminate()
            let deadline = Date().addingTimeInterval(5)
            while !instance.isTerminated && Date() < deadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
        }
        exit(1)
    }
}
func waitFor(_ message: String, _ condition: () -> Bool) {
    let deadline = Date().addingTimeInterval(15)
    while !condition() && Date() < deadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
    require(condition(), message)
}

require(CommandLine.arguments.count == 2, "pass a disposable devtools app bundle")
let app = URL(fileURLWithPath: CommandLine.arguments[1]).resolvingSymlinksInPath()
let temp = URL(fileURLWithPath: "/tmp").resolvingSymlinksInPath()
require(Bundle(url: app)?.bundleIdentifier?.hasPrefix("local.quietmd.finder-test.") == true, "use a disposable local.quietmd.finder-test.* bundle ID")
let binary = try Data(contentsOf: app.appendingPathComponent("Contents/MacOS/quiet_md"))
require(binary.range(of: Data("devtools: cannot read".utf8)) != nil, "use a devtools build so tests do not activate windows")
let dir = temp.appendingPathComponent("quiet-md-open-files-" + UUID().uuidString)
try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
defer { try? FileManager.default.removeItem(at: dir) }
let files = ["冷启动 空格 #%?.md", "warm.MDX", "multi.md"].map { dir.appendingPathComponent($0) }
for file in files { try "# Finder regression\n\nOriginal content.\n".write(to: file, atomically: true, encoding: .utf8) }
let originals = try files.map { try Data(contentsOf: $0) }
let defaultMD = NSWorkspace.shared.urlForApplication(toOpen: files[0])
require(LSRegisterURL(app as CFURL, true) == noErr, "LaunchServices registration")
for file in files {
    require(NSWorkspace.shared.urlsForApplications(toOpen: file).contains { $0.resolvingSymlinksInPath() == app }, "Open With candidate for \(file.lastPathComponent)")
}
require(NSWorkspace.shared.urlForApplication(toOpen: files[0]) == defaultMD, "existing Markdown default must stay unchanged")

let script = dir.appendingPathComponent("idle.txt")
try "wait 120000\nquit\n".write(to: script, atomically: true, encoding: .utf8)
let config = NSWorkspace.OpenConfiguration()
config.activates = false
config.createsNewApplicationInstance = true
config.environment = ["QUIET_MD_HOME": dir.appendingPathComponent("library").path,
                      "QUIET_MD_LOG_DIR": dir.appendingPathComponent("logs").path,
                      "FDL_SCRIPT": script.path]
var running: NSRunningApplication?
var launchError: Error?
var launched = false
NSWorkspace.shared.open([files[0]], withApplicationAt: app, configuration: config) { instance, error in
    running = instance; launchError = error; launched = true
}
waitFor("cold launch completed") { launched }
require(launchError == nil && running != nil, "cold launch: \(String(describing: launchError))")
var instance = running!
runningTest = instance
defer { if !instance.isTerminated { instance.terminate() } }
require(instance.executableURL?.resolvingSymlinksInPath() == app.appendingPathComponent("Contents/MacOS/quiet_md"), "test must target this bundle")
func titles() -> [String] {
    let windows = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.filter { ($0[kCGWindowOwnerPID as String] as? Int32) == instance.processIdentifier }
        .compactMap { $0[kCGWindowName as String] as? String }
}
waitFor("cold Finder event opens the decoded Markdown path") { titles().contains(files[0].lastPathComponent + " — Quiet.md") }
require(titles().filter { !$0.isEmpty }.count == 1, "cold file launch must not open the library home window")
print("PASS: registered md/mdx; cold launch opens only the requested file")

let warm = NSWorkspace.OpenConfiguration()
warm.activates = false
var delivered = false
NSWorkspace.shared.open(Array(files.dropFirst()), withApplicationAt: app, configuration: warm) { same, error in
    require(error == nil && same?.processIdentifier == instance.processIdentifier, "warm event must reuse the running app")
    delivered = true
}
waitFor("warm multi-file event") { delivered && files.allSatisfy { titles().contains($0.lastPathComponent + " — Quiet.md") } }
require(titles().filter { $0.hasSuffix(" — Quiet.md") }.count == files.count, "one editor window per file")
for (file, bytes) in zip(files, originals) { require(try Data(contentsOf: file) == bytes, "opening must not rewrite the file") }
require(NSWorkspace.shared.urlForApplication(toOpen: files[0]) == defaultMD, "Markdown default after opening")
require(instance.terminate(), "graceful test app quit")
waitFor("test app quit") { instance.isTerminated }
print("PASS: warm multi-file md/mdx event; original bytes and Markdown default preserved")

func launchWithArguments(_ args: [String]) -> NSRunningApplication {
    config.arguments = args
    var result: NSRunningApplication?
    var failure: Error?
    var done = false
    NSWorkspace.shared.openApplication(at: app, configuration: config) { application, error in
        result = application; failure = error; done = true
    }
    waitFor("application launch") { done }
    require(failure == nil && result != nil, "launch: \(String(describing: failure))")
    runningTest = result
    return result!
}
func quitInstance() {
    require(instance.terminate(), "graceful test app quit")
    waitFor("test app quit") { instance.isTerminated }
}

instance = launchWithArguments([files[0].path])
waitFor("command-line file opens") { titles().contains(files[0].lastPathComponent + " — Quiet.md") }
require(titles().filter { !$0.isEmpty }.count == 1, "command-line file launch must not open the home window")
quitInstance()
print("PASS: command-line file launch opens only the requested file")

instance = launchWithArguments([])
waitFor("ordinary launch opens the library home") { titles().contains("Quiet.md") }
require(!titles().contains { $0.hasSuffix(" — Quiet.md") }, "ordinary launch must not reopen test files")
quitInstance()
print("PASS: ordinary application launch still opens the library home")
