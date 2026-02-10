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

    private(set) var window: NSWindow?

    private var imageSourcePopup: NSPopUpButton?
    private var apiKeyField: NSTextField?
    private var apiKeyLabel: NSTextField?
    private var apiKeyHint: NSTextField?
    private var apiKeyGetButton: NSButton?
    private var cacheCountLabel: NSTextField?
    private var clearCacheButton: NSButton?
    private var fetchButton: NSButton?
    private var statusLabel: NSTextField?

    private let imageSources = [
        ("bundled", "Bundled Images"),
        ("nasa_images", "NASA Image Library"),
        ("apod", "NASA APOD (requires API key)"),
        ("apod_with_fallback", "APOD with Fallback"),
    ]

    // MARK: - Initialization

    override init() {
        super.init()
        setupWindow()
    }

    // MARK: - Setup

    private func setupWindow() {
        let contentRect = NSRect(x: 0, y: 0, width: 420, height: 300)
        let panel = NSPanel(
            contentRect: contentRect,
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        panel.title = "Spacesaver Settings"
        panel.isFloatingPanel = true
        panel.becomesKeyOnlyIfNeeded = false
        window = panel

        let contentView = NSView(frame: contentRect)
        panel.contentView = contentView

        var yOffset: CGFloat = 255

        // Title
        let titleLabel = createLabel("Spacesaver Settings", bold: true)
        titleLabel.frame = NSRect(x: 20, y: yOffset, width: 380, height: 24)
        titleLabel.font = NSFont.boldSystemFont(ofSize: 16)
        contentView.addSubview(titleLabel)
        yOffset -= 40

        // Image Source section
        let sourceLabel = createLabel("Image Source:")
        sourceLabel.frame = NSRect(x: 20, y: yOffset, width: 120, height: 22)
        contentView.addSubview(sourceLabel)

        let popup = NSPopUpButton(frame: NSRect(x: 140, y: yOffset - 2, width: 250, height: 26))
        popup.removeAllItems()
        for (_, displayName) in imageSources {
            popup.addItem(withTitle: displayName)
        }
        popup.target = self
        popup.action = #selector(imageSourceChanged)
        contentView.addSubview(popup)
        imageSourcePopup = popup
        yOffset -= 35

        // API Key section
        let apiLabel = createLabel("NASA API Key:")
        apiLabel.frame = NSRect(x: 20, y: yOffset, width: 120, height: 22)
        contentView.addSubview(apiLabel)
        apiKeyLabel = apiLabel

        let apiField = NSTextField(frame: NSRect(x: 140, y: yOffset, width: 180, height: 22))
        apiField.placeholderString = "DEMO_KEY"
        apiField.target = self
        apiField.action = #selector(saveApiKey)
        contentView.addSubview(apiField)
        apiKeyField = apiField

        let apiHelpButton = NSButton(frame: NSRect(x: 325, y: yOffset, width: 65, height: 22))
        apiHelpButton.title = "Get Key"
        apiHelpButton.bezelStyle = .inline
        apiHelpButton.target = self
        apiHelpButton.action = #selector(openApiPage)
        contentView.addSubview(apiHelpButton)
        apiKeyGetButton = apiHelpButton

        yOffset -= 25

        // API Key hint
        let hintLabel = createLabel("Get a free API key at api.nasa.gov", bold: false)
        hintLabel.frame = NSRect(x: 140, y: yOffset, width: 240, height: 18)
        hintLabel.font = NSFont.systemFont(ofSize: 10)
        hintLabel.textColor = .secondaryLabelColor
        contentView.addSubview(hintLabel)
        apiKeyHint = hintLabel
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
        status.frame = NSRect(x: 20, y: yOffset, width: 380, height: 22)
        status.alignment = .center
        status.textColor = .secondaryLabelColor
        contentView.addSubview(status)
        statusLabel = status

        // OK button
        let okButton = NSButton(frame: NSRect(x: 310, y: 15, width: 80, height: 28))
        okButton.title = "OK"
        okButton.bezelStyle = .rounded
        okButton.keyEquivalent = "\r"
        okButton.target = self
        okButton.action = #selector(closeSheet)
        contentView.addSubview(okButton)

        // Load current values
        loadCurrentValues()
    }

    private func loadCurrentValues() {
        refreshCacheCount()

        // Load current image source
        if let sourcePtr = spacesaver_get_image_source() {
            let source = String(cString: sourcePtr)
            spacesaver_free_string(sourcePtr)

            if let index = imageSources.firstIndex(where: { $0.0 == source }) {
                imageSourcePopup?.selectItem(at: index)
            }
        }

        updateApiKeyVisibility()
    }

    private func updateApiKeyVisibility() {
        let selectedIndex = imageSourcePopup?.indexOfSelectedItem ?? 0
        let source = imageSources[selectedIndex].0
        let needsApiKey = (source == "apod" || source == "apod_with_fallback")

        apiKeyField?.isHidden = !needsApiKey
        apiKeyLabel?.isHidden = !needsApiKey
        apiKeyHint?.isHidden = !needsApiKey
        apiKeyGetButton?.isHidden = !needsApiKey
        fetchButton?.isHidden = (source == "bundled")
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

    @objc private func imageSourceChanged() {
        let selectedIndex = imageSourcePopup?.indexOfSelectedItem ?? 0
        let source = imageSources[selectedIndex].0

        let cSource = source.cString(using: .utf8)
        var result = spacesaver_set_image_source(cSource)
        defer { spacesaver_free_result(&result) }

        if result.success {
            statusLabel?.stringValue = "Image source updated"
            statusLabel?.textColor = .systemGreen
        } else {
            statusLabel?.stringValue = "Failed to update image source"
            statusLabel?.textColor = .systemRed
        }

        updateApiKeyVisibility()

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { [weak self] in
            self?.statusLabel?.stringValue = ""
        }
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

        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
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
        if let parent = window.sheetParent {
            parent.endSheet(window)
        } else {
            window.close()
        }
    }
}
