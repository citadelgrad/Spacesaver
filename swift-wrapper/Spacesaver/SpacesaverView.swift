//
//  SpacesaverView.swift
//  Spacesaver
//
//  A macOS screen saver that displays NASA Astronomy Picture of the Day images.
//

import ScreenSaver
import AppKit

@objc(SpacesaverView)
class SpacesaverView: ScreenSaverView {

    // MARK: - Properties

    private var currentImageView: NSImageView?
    private var nextImageView: NSImageView?
    private var titleLabel: NSTextField?
    private var dateLabel: NSTextField?
    private var copyrightLabel: NSTextField?
    private var loadingIndicator: NSProgressIndicator?

    private var isInitialized = false
    private var isTransitioning = false
    private var transitionTimer: Timer?

    // Configuration
    private let transitionInterval: TimeInterval = 30.0
    private let transitionDuration: TimeInterval = 2.0
    private let showTitle = true
    private let prefetchCount: Int32 = 10

    // MARK: - Initialization

    override init?(frame: NSRect, isPreview: Bool) {
        NSLog("Spacesaver: init(frame:isPreview:) called, preview=\(isPreview)")
        super.init(frame: frame, isPreview: isPreview)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        NSLog("Spacesaver: commonInit() started")
        animationTimeInterval = 1.0 / 30.0 // 30 FPS
        wantsLayer = true
        layer?.backgroundColor = NSColor.black.cgColor

        NSLog("Spacesaver: calling setupViews()")
        setupViews()
        NSLog("Spacesaver: calling initializeRustLibrary()")
        initializeRustLibrary()
        NSLog("Spacesaver: commonInit() completed")
    }

    deinit {
        transitionTimer?.invalidate()
        spacesaver_shutdown()
    }

    // MARK: - Setup

    private func setupViews() {
        // Current image view (fills entire screen)
        let imageView = NSImageView(frame: bounds)
        imageView.imageScaling = .scaleProportionallyUpOrDown
        imageView.autoresizingMask = [.width, .height]
        imageView.wantsLayer = true
        addSubview(imageView)
        currentImageView = imageView

        // Next image view for transitions (initially hidden)
        let nextView = NSImageView(frame: bounds)
        nextView.imageScaling = .scaleProportionallyUpOrDown
        nextView.autoresizingMask = [.width, .height]
        nextView.wantsLayer = true
        nextView.alphaValue = 0
        addSubview(nextView)
        nextImageView = nextView

        // Title label at bottom
        if showTitle {
            let title = createLabel()
            title.font = NSFont.systemFont(ofSize: isPreview ? 12 : 28, weight: .medium)
            title.translatesAutoresizingMaskIntoConstraints = false
            addSubview(title)
            titleLabel = title

            let date = createLabel()
            date.font = NSFont.systemFont(ofSize: isPreview ? 8 : 16, weight: .regular)
            date.textColor = NSColor.white.withAlphaComponent(0.8)
            date.translatesAutoresizingMaskIntoConstraints = false
            addSubview(date)
            dateLabel = date

            let copyright = createLabel()
            copyright.font = NSFont.systemFont(ofSize: isPreview ? 6 : 12, weight: .light)
            copyright.textColor = NSColor.white.withAlphaComponent(0.6)
            copyright.translatesAutoresizingMaskIntoConstraints = false
            addSubview(copyright)
            copyrightLabel = copyright

            // Layout constraints
            NSLayoutConstraint.activate([
                title.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 20),
                title.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -20),
                title.bottomAnchor.constraint(equalTo: date.topAnchor, constant: -4),

                date.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 20),
                date.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -20),
                date.bottomAnchor.constraint(equalTo: copyright.topAnchor, constant: -2),

                copyright.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 20),
                copyright.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -20),
                copyright.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -20),
            ])
        }

        // Loading indicator
        let indicator = NSProgressIndicator(frame: NSRect(x: 0, y: 0, width: 32, height: 32))
        indicator.style = .spinning
        indicator.isIndeterminate = true
        indicator.translatesAutoresizingMaskIntoConstraints = false
        addSubview(indicator)
        loadingIndicator = indicator

        NSLayoutConstraint.activate([
            indicator.centerXAnchor.constraint(equalTo: centerXAnchor),
            indicator.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
    }

    private func createLabel() -> NSTextField {
        let label = NSTextField(labelWithString: "")
        label.textColor = .white
        label.backgroundColor = .clear
        label.isBezeled = false
        label.isEditable = false
        label.isSelectable = false
        label.drawsBackground = false
        label.cell?.backgroundStyle = .dark

        // Add shadow for readability
        let shadow = NSShadow()
        shadow.shadowColor = NSColor.black.withAlphaComponent(0.8)
        shadow.shadowBlurRadius = 4
        shadow.shadowOffset = NSSize(width: 1, height: -1)
        label.shadow = shadow

        return label
    }

    // MARK: - Rust Library Integration

    private func initializeRustLibrary() {
        let result = spacesaver_init(nil)
        if result.success {
            isInitialized = true

            // Check if we have cached images
            let cachedCount = spacesaver_cached_count()
            if cachedCount > 0 {
                // Load first image
                loadNextImage()
            } else {
                // Start fetching images
                showLoading(true)
                fetchImagesInBackground()
            }
        } else {
            if let errorPtr = result.error {
                let error = String(cString: errorPtr)
                NSLog("Spacesaver: Failed to initialize: \(error)")
                spacesaver_free_string(errorPtr)
            }
        }
    }

    private func fetchImagesInBackground() {
        // Use userInitiated QoS - background QoS doesn't execute reliably in screensaver context
        let count = self.prefetchCount
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            NSLog("Spacesaver: Starting fetch of \(count) images")
            let fetched = spacesaver_fetch_random(count)
            NSLog("Spacesaver: Fetch returned \(fetched) images")

            DispatchQueue.main.async {
                self?.showLoading(false)
                if fetched > 0 {
                    self?.loadNextImage()
                    self?.startTransitionTimer()
                } else {
                    NSLog("Spacesaver: Failed to fetch images, will retry on next animation start")
                }
            }
        }
    }

    private func showLoading(_ show: Bool) {
        if show {
            loadingIndicator?.startAnimation(nil)
            loadingIndicator?.isHidden = false
        } else {
            loadingIndicator?.stopAnimation(nil)
            loadingIndicator?.isHidden = true
        }
    }

    // MARK: - Image Loading

    private func loadNextImage() {
        guard isInitialized else { return }

        var imageInfo = spacesaver_next_image()
        defer {
            spacesaver_free_image(&imageInfo)
        }

        guard let pathPtr = imageInfo.path else {
            NSLog("Spacesaver: No image available")
            return
        }

        let path = String(cString: pathPtr)
        let url = URL(fileURLWithPath: path)

        guard let image = NSImage(contentsOf: url) else {
            NSLog("Spacesaver: Failed to load image: \(path)")
            return
        }

        // Update labels
        if let titlePtr = imageInfo.title {
            titleLabel?.stringValue = String(cString: titlePtr)
        }

        if let datePtr = imageInfo.date {
            dateLabel?.stringValue = formatDate(String(cString: datePtr))
        }

        if let copyrightPtr = imageInfo.copyright {
            copyrightLabel?.stringValue = "© \(String(cString: copyrightPtr))"
            copyrightLabel?.isHidden = false
        } else {
            copyrightLabel?.stringValue = "NASA"
            copyrightLabel?.isHidden = false
        }

        // Transition to new image
        transitionToImage(image)
    }

    private func formatDate(_ dateString: String) -> String {
        let inputFormatter = DateFormatter()
        inputFormatter.dateFormat = "yyyy-MM-dd"

        let outputFormatter = DateFormatter()
        outputFormatter.dateStyle = .long
        outputFormatter.timeStyle = .none

        if let date = inputFormatter.date(from: dateString) {
            return outputFormatter.string(from: date)
        }
        return dateString
    }

    private func transitionToImage(_ image: NSImage) {
        guard !isTransitioning else { return }
        isTransitioning = true

        nextImageView?.image = image
        nextImageView?.alphaValue = 0

        NSAnimationContext.runAnimationGroup({ context in
            context.duration = transitionDuration
            context.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)

            nextImageView?.animator().alphaValue = 1
            currentImageView?.animator().alphaValue = 0
        }, completionHandler: { [weak self] in
            // Swap views
            let temp = self?.currentImageView
            self?.currentImageView = self?.nextImageView
            self?.nextImageView = temp

            self?.nextImageView?.alphaValue = 0
            self?.currentImageView?.alphaValue = 1

            self?.isTransitioning = false
        })
    }

    // MARK: - Timer

    private func startTransitionTimer() {
        transitionTimer?.invalidate()
        transitionTimer = Timer.scheduledTimer(withTimeInterval: transitionInterval, repeats: true) { [weak self] _ in
            self?.loadNextImage()
        }
    }

    // MARK: - ScreenSaverView Overrides

    override func startAnimation() {
        super.startAnimation()

        if !isInitialized {
            initializeRustLibrary()
        }

        startTransitionTimer()

        // Start background prefetching
        spacesaver_start_prefetch(prefetchCount)
    }

    override func stopAnimation() {
        super.stopAnimation()
        transitionTimer?.invalidate()
        transitionTimer = nil
    }

    override func draw(_ rect: NSRect) {
        NSColor.black.setFill()
        rect.fill()
        super.draw(rect)
    }

    override func animateOneFrame() {
        // Animation is handled by the timer and transitions
        // This is called at animationTimeInterval rate
    }

    override var hasConfigureSheet: Bool {
        return true
    }

    override var configureSheet: NSWindow? {
        return ConfigureSheetController.shared.window
    }
}
