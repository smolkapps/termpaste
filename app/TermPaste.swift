// TermPaste — macOS menu-bar app. A thin AppKit shell around the deterministic
// `termpaste` CLI: it watches the clipboard via NSPasteboard.changeCount and, on a
// new copy, asks the bundled `termpaste` binary to clean it. All cleaning logic and
// the terminal-only pre-gate live in the tested Rust core. See spec-menubar.md.
//
// Entry point is @main (explicit) — a file that mixes type declarations with bare
// top-level statements does not reliably run the top-level code as main under swiftc.
import AppKit
import Foundation

func tpLog(_ msg: String) {
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
        let t = Timer(timeInterval: 0.3, repeats: true) { [weak self] _ in self?.poll() }
        RunLoop.main.add(t, forMode: .common)
        timer = t
        tpLog("watching")
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
        tpLog("clipboard changed → clean (\(cleanEverything ? "all" : "terminal-only"))")
        runClean()
        lastChangeCount = NSPasteboard.general.changeCount
    }

    private func runClean() {
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: termpastePath)
        proc.arguments = [cleanEverything ? "--clipboard" : "--clipboard-terminal"]
        do {
            try proc.run()
            proc.waitUntilExit()
        } catch {}
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

@main
enum TermPasteMain {
    static func main() {
        tpLog("main: start")
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory) // menu-bar only, no Dock icon (LSUIElement)
        let controller = Controller()
        _ = controller // retain for process lifetime
        app.run()
    }
}
