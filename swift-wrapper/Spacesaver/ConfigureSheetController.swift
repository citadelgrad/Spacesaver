//
//  ConfigureSheetController.swift
//  Spacesaver
//
//  Configuration sheet for the Spacesaver screen saver.
//

import AppKit
import ScreenSaver

class ConfigureSheetController: NSObject {

    // MARK: - Singleton

    static let shared = ConfigureSheetController()

    // MARK: - Properties

    var window: NSWindow?

    private var apiKeyField: NSTextField?
    private var cacheCountLabel: NSTextField?
    private var clearCacheButton: NSButton?
    private var fetchButton: NSButton?
    private var statusLabel: NSTextField?

    // MARK: - Initialization

    override init() {
        super.init()
        setupWindow()
    }

    // MARK: - Setup

    private func setupWindow() {
        let contentRect = NSRect(x: 0, y: 0, width: 400, height: 250)
        window = NSWindow(
            contentRect: contentRect,
            styleMask: [.titled],
            backing: .buffered,
            defer: true
        )
        window?.title = "Spacesaver Settings"

        let contentView = NSView(frame: contentRect)
        window?.contentView = contentView

        var yOffset: CGFloat = 200

        // Title
        let titleLabel = createLabel("NASA Spacesaver Settings", bold: true)
        titleLabel.frame = NSRect(x: 20, y: yOffset, width: 360, height: 24)
        titleLabel.font = NSFont.boldSystemFont(ofSize: 16)
        contentView.addSubview(titleLabel)
        yOffset -= 40

        // API Key section
        let apiLabel = createLabel("NASA API Key:")
        apiLabel.frame = NSRect(x: 20, y: yOffset, width: 120, height: 22)
        contentView.addSubview(apiLabel)

        let apiField = NSTextField(frame: NSRect(x: 140, y: yOffset, width: 180, height: 22))
        apiField.placeholderString = "DEMO_KEY"
        apiField.target = self
        apiField.action = #selector(saveApiKey)
        contentView.addSubview(apiField)
        apiKeyField = apiField

        let apiHelpButton = NSButton(frame: NSRect(x: 325, y: yOffset, width: 60, height: 22))
        apiHelpButton.title = "Get Key"
        apiHelpButton.bezelStyle = .inline
        apiHelpButton.target = self
        apiHelpButton.action = #selector(openApiPage)
        contentView.addSubview(apiHelpButton)

        yOffset -= 30

        // API Key hint
        let hintLabel = createLabel("Get a free API key at api.nasa.gov", bold: false)
        hintLabel.frame = NSRect(x: 140, y: yOffset, width: 240, height: 18)
        hintLabel.font = NSFont.systemFont(ofSize: 10)
        hintLabel.textColor = .secondaryLabelColor
        contentView.addSubview(hintLabel)
        yOffset -= 35

        // Cache section
        let cacheLabel = createLabel("Cached Images:")
        cacheLabel.frame = NSRect(x: 20, y: yOffset, width: 120, height: 22)
        contentView.addSubview(cacheLabel)

        let countLabel = createLabel("0 images")
        countLabel.frame = NSRect(x: 140, y: yOffset, width: 100, height: 22)
        contentView.addSubview(countLabel)
        cacheCountLabel = countLabel

        let clearButton = NSButton(frame: NSRect(x: 250, y: yOffset, width: 80, height: 22))
        clearButton.title = "Clear"
        clearButton.bezelStyle = .rounded
        clearButton.target = self
        clearButton.action = #selector(clearCache)
        contentView.addSubview(clearButton)
        clearCacheButton = clearButton

        yOffset -= 35

        // Fetch button
        let fetchBtn = NSButton(frame: NSRect(x: 140, y: yOffset, width: 150, height: 28))
        fetchBtn.title = "Fetch New Images"
        fetchBtn.bezelStyle = .rounded
        fetchBtn.target = self
        fetchBtn.action = #selector(fetchImages)
        contentView.addSubview(fetchBtn)
        fetchButton = fetchBtn

        yOffset -= 30

        // Status label
        let status = createLabel("")
        status.frame = NSRect(x: 20, y: yOffset, width: 360, height: 22)
        status.alignment = .center
        status.textColor = .secondaryLabelColor
        contentView.addSubview(status)
        statusLabel = status

        yOffset -= 40

        // OK/Cancel buttons
        let okButton = NSButton(frame: NSRect(x: 290, y: 15, width: 80, height: 28))
        okButton.title = "OK"
        okButton.bezelStyle = .rounded
        okButton.keyEquivalent = "\r"
        okButton.target = self
        okButton.action = #selector(closeSheet)
        contentView.addSubview(okButton)

        let cancelButton = NSButton(frame: NSRect(x: 200, y: 15, width: 80, height: 28))
        cancelButton.title = "Cancel"
        cancelButton.bezelStyle = .rounded
        cancelButton.keyEquivalent = "\u{1b}"
        cancelButton.target = self
        cancelButton.action = #selector(closeSheet)
        contentView.addSubview(cancelButton)

        // Load current values
        refreshCacheCount()
    }

    private func createLabel(_ text: String, bold: Bool = false) -> NSTextField {
        let label = NSTextField(labelWithString: text)
        label.isBezeled = false
        label.drawsBackground = false
        label.isEditable = false
        label.isSelectable = false
        if bold {
            label.font = NSFont.boldSystemFont(ofSize: NSFont.systemFontSize)
        }
        return label
    }

    // MARK: - Actions

    private func refreshCacheCount() {
        let count = spacesaver_cached_count()
        cacheCountLabel?.stringValue = "\(count) images"
    }

    @objc private func saveApiKey() {
        guard let key = apiKeyField?.stringValue, !key.isEmpty else { return }

        let cKey = key.cString(using: .utf8)
        var result = spacesaver_set_api_key(cKey)
        defer { spacesaver_free_result(&result) }

        if result.success {
            statusLabel?.stringValue = "API key saved"
            statusLabel?.textColor = .systemGreen
        } else {
            statusLabel?.stringValue = "Failed to save API key"
            statusLabel?.textColor = .systemRed
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { [weak self] in
            self?.statusLabel?.stringValue = ""
        }
    }

    @objc private func openApiPage() {
        if let url = URL(string: "https://api.nasa.gov/") {
            NSWorkspace.shared.open(url)
        }
    }

    @objc private func clearCache() {
        var result = spacesaver_clear_cache()
        defer { spacesaver_free_result(&result) }

        if result.success {
            statusLabel?.stringValue = "Cache cleared"
            statusLabel?.textColor = .systemGreen
            refreshCacheCount()
        } else {
            statusLabel?.stringValue = "Failed to clear cache"
            statusLabel?.textColor = .systemRed
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { [weak self] in
            self?.statusLabel?.stringValue = ""
        }
    }

    @objc private func fetchImages() {
        statusLabel?.stringValue = "Fetching images..."
        statusLabel?.textColor = .secondaryLabelColor
        fetchButton?.isEnabled = false

        DispatchQueue.global(qos: .background).async { [weak self] in
            let count = spacesaver_fetch_random(10)

            DispatchQueue.main.async {
                self?.fetchButton?.isEnabled = true
                if count > 0 {
                    self?.statusLabel?.stringValue = "Fetched \(count) images"
                    self?.statusLabel?.textColor = .systemGreen
                } else {
                    self?.statusLabel?.stringValue = "Failed to fetch images"
                    self?.statusLabel?.textColor = .systemRed
                }
                self?.refreshCacheCount()

                DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                    self?.statusLabel?.stringValue = ""
                }
            }
        }
    }

    @objc private func closeSheet() {
        guard let window = window else { return }
        window.sheetParent?.endSheet(window)
    }
}
