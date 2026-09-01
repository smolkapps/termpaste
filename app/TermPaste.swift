// TermPaste — macOS menu-bar app. A thin AppKit shell around the deterministic
// `termpaste` CLI: it watches the clipboard via NSPasteboard.changeCount and, on a
// new copy, asks the bundled `termpaste` binary to clean it. All cleaning logic and
// the terminal-only pre-gate live in the tested Rust core — this file only decides
// *when* to run it and provides the menu-bar UX. See spec-menubar.md.
//
// Setup happens in Controller.init() (before NSApp.run()), not in an
// NSApplicationDelegate callback — a manually-constructed NSApplication does not
// reliably deliver applicationDidFinishLaunching, which would leave the run loop
// with no timer and exit immediately.
import AppKit
import Foundation

final class Controller: NSObject {
    private var statusItem: NSStatusItem!
    private var timer: Timer?
    private var lastChangeCount = NSPasteboard.general.changeCount
    private var enabled = true
    private var cleanEverything = false // false = terminal-only (default, per spec)

    private lazy var termpastePath: String = {
        let bundled = Bundle.main.bundlePath + "/Contents/MacOS/termpaste"
        if FileManager.default.isExecutableFile(atPath: bundled) {
            return bundled
        }
        return FileManager.default.homeDirectoryForCurrentUser.path + "/.cargo/bin/termpaste"
    }()

    override init() {
        super.init()
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.title = "✂︎"
        rebuildMenu()
        // Common-mode timer so it keeps firing during menu tracking.
        let t = Timer(timeInterval: 0.3, repeats: true) { [weak self] _ in self?.poll() }
        RunLoop.main.add(t, forMode: .common)
        timer = t
        log("launched")
    }

    private func log(_ msg: String) {
        let path = FileManager.default.homeDirectoryForCurrentUser.path
            + "/Library/Logs/termpaste-app.log"
        guard let data = "termpaste-app: \(msg)\n".data(using: .utf8) else { return }
        if let fh = FileHandle(forWritingAtPath: path) {
            fh.seekToEndOfFile()
            fh.write(data)
            fh.closeFile()
        } else {
            try? data.write(to: URL(fileURLWithPath: path))
        }
    }

    private func rebuildMenu() {
        let menu = NSMenu()
        let status = NSMenuItem(
            title: enabled ? "Watching the clipboard" : "Paused", action: nil, keyEquivalent: ""
        )
        status.isEnabled = false
        menu.addItem(status)
        menu.addItem(.separator())
        menu.addItem(item("Enable cleaning", #selector(toggleEnabled), on: enabled))
        menu.addItem(
            item(
                "Clean everything (not just terminal output)", #selector(toggleCleanEverything),
                on: cleanEverything
            )
        )
        menu.addItem(item("Clean clipboard now", #selector(cleanNow), on: false))
        menu.addItem(.separator())
        menu.addItem(
            NSMenuItem(
                title: "Quit TermPaste", action: #selector(NSApplication.terminate(_:)),
                keyEquivalent: "q"
            )
        )
        statusItem.menu = menu
    }

    private func item(_ title: String, _ action: Selector, on: Bool) -> NSMenuItem {
        let mi = NSMenuItem(title: title, action: action, keyEquivalent: "")
        mi.target = self
        mi.state = on ? .on : .off
        return mi
    }

    private func poll() {
        guard enabled else { return }
        let cc = NSPasteboard.general.changeCount
        if cc == lastChangeCount {
            return
        }
        log("clipboard changed → clean (\(cleanEverything ? "all" : "terminal-only"))")
        runClean()
        // Adopt whatever count exists after our own write so we never re-trigger on it.
        lastChangeCount = NSPasteboard.general.changeCount
    }

    private func runClean() {
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: termpastePath)
        proc.arguments = [cleanEverything ? "--clipboard" : "--clipboard-terminal"]
        do {
            try proc.run()
            proc.waitUntilExit()
        } catch {
            // Missing/unrunnable binary: fail quiet, keep watching.
        }
    }

    @objc private func toggleEnabled() {
        enabled.toggle()
        lastChangeCount = NSPasteboard.general.changeCount
        rebuildMenu()
    }

    @objc private func toggleCleanEverything() {
        cleanEverything.toggle()
        rebuildMenu()
    }

    @objc private func cleanNow() {
        runClean()
        lastChangeCount = NSPasteboard.general.changeCount
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory) // menu-bar only, no Dock icon (LSUIElement)
let controller = Controller()
_ = controller // keep it alive for the process lifetime
app.run()
