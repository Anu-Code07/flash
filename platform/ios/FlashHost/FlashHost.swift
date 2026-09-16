// Flash iOS native renderer — UIKit mapping (stable widget IDs from flash-stl).
// Wire: Rust static lib calls flash_host_register() at launch.

import UIKit

// Widget kind IDs: stl/flash-stl/src/widgets/*.rs

enum FlashPropKey: UInt16 {
    case text = 0
    case title = 1
    case value = 2
    case src = 3
}

// MARK: - Host

@objc public final class FlashHost: NSObject {
    public static let shared = FlashHost()

    private var views: [UInt32: UIView] = [:]
    private var handlers: [UInt32: UInt32] = [:]
    public weak var rootView: UIView?

    private override init() {
        super.init()
    }

    /// Register C vtable with Rust runtime (call from AppDelegate).
    public func registerWithRust() {
        flash_host_register(FlashHostVTable(
            create: { handle, kind in FlashHost.shared.create(handle: handle, kind: kind) },
            set_prop: { handle, key, ptr, len in
                FlashHost.shared.setProp(handle: handle, key: key, ptr: ptr, len: len)
            },
            insert_child: { parent, child, index in
                FlashHost.shared.insertChild(parent: parent, child: child, index: index)
            },
            remove: { handle in FlashHost.shared.remove(handle: handle) },
            set_handler: { handle, handlerId in
                FlashHost.shared.handlers[handle] = handlerId
            },
            set_frame: { handle, x, y, width, height in
                FlashHost.shared.setFrame(handle: handle, x: x, y: y, width: width, height: height)
            },
            commit: { FlashHost.shared.layoutIfNeeded() }
        ))
    }

    /// Batch apply encoded ops from Rust (preferred — one JNI/FFI crossing per frame).
    public func applyOps(_ data: Data) {
        flash_host_apply_ops(data.withUnsafeBytes { $0.baseAddress! }, UInt32(data.count))
    }

    // MARK: - Op handlers

    func create(handle: UInt32, kind: UInt16) {
        let view = makeView(kind: kind)
        views[handle] = view
        if rootView == nil, let container = view as? UIStackView {
            rootView = container
        }
    }

    func setProp(handle: UInt32, key: UInt16, ptr: UnsafePointer<UInt8>?, len: UInt32) {
        guard let view = views[handle], let ptr = ptr else { return }
        let text = String(bytes: UnsafeBufferPointer(start: ptr, count: Int(len)), encoding: .utf8) ?? ""
        switch FlashPropKey(rawValue: key) {
        case .text:
            if let label = view as? UILabel { label.text = text }
            else if let field = view as? UITextField { field.text = text }
            else if let iv = view as? UIImageView, let img = UIImage(systemName: text) {
                iv.image = img
            }
            else if view.accessibilityIdentifier == "flash-badge" {
                (view.subviews.first as? UILabel)?.text = text
            }
        case .title:
            if let button = view as? UIButton { button.setTitle(text, for: .normal) }
        case .value:
            if let field = view as? UITextField { field.text = text }
            else if let sw = view as? UISwitch { sw.isOn = text == "1" || text == "true" }
            else if let progress = view as? UIProgressView { progress.progress = Float(text) ?? 0 }
            else if let check = view as? UIButton, check.accessibilityIdentifier == "flash-checkbox" {
                check.isSelected = text == "1" || text == "true"
            }
        case .src:
            if let iv = view as? UIImageView, let url = URL(string: text) {
                URLSession.shared.dataTask(with: url) { data, _, _ in
                    guard let data = data else { return }
                    DispatchQueue.main.async { iv.image = UIImage(data: data) }
                }.resume()
            }
        default:
            break
        }
    }

    func insertChild(parent: UInt32, child: UInt32, index: UInt32) {
        guard let childView = views[child] else { return }
        if let parentView = views[parent] {
            insert(childView, into: parentView, at: Int(index))
        } else if parent == UInt32.max, let root = rootView {
            insert(childView, into: root, at: Int(index))
        }
    }

    func remove(handle: UInt32) {
        views[handle]?.removeFromSuperview()
        views.removeValue(forKey: handle)
    }

    func setFrame(handle: UInt32, x: Float, y: Float, width: Float, height: Float) {
        guard let view = views[handle] else { return }
        view.frame = CGRect(x: CGFloat(x), y: CGFloat(y), width: CGFloat(width), height: CGFloat(height))
        view.translatesAutoresizingMaskIntoConstraints = true
    }

    func layoutIfNeeded() {
        rootView?.setNeedsLayout()
        rootView?.layoutIfNeeded()
    }

    // MARK: - View factory

    private func makeView(kind: UInt16) -> UIView {
        switch kind {
        case 0: // Text
            let label = UILabel()
            label.numberOfLines = 0
            return label
        case 1: // Button
            let button = UIButton(type: .system)
            button.addTarget(self, action: #selector(buttonTapped(_:)), for: .touchUpInside)
            return button
        case 2: // Column
            let stack = UIStackView()
            stack.axis = .vertical
            stack.alignment = .fill
            stack.spacing = 8
            return stack
        case 3: // Row
            let stack = UIStackView()
            stack.axis = .horizontal
            stack.alignment = .center
            stack.spacing = 8
            return stack
        case 4: // Stack
            return UIView()
        case 5: // Image
            let iv = UIImageView()
            iv.contentMode = .scaleAspectFit
            iv.clipsToBounds = true
            return iv
        case 6: // TextField
            return UITextField()
        case 7: // ScrollView
            let sv = UIScrollView()
            let content = UIStackView()
            content.axis = .vertical
            content.translatesAutoresizingMaskIntoConstraints = false
            sv.addSubview(content)
            NSLayoutConstraint.activate([
                content.topAnchor.constraint(equalTo: sv.topAnchor),
                content.leadingAnchor.constraint(equalTo: sv.leadingAnchor),
                content.trailingAnchor.constraint(equalTo: sv.trailingAnchor),
                content.bottomAnchor.constraint(equalTo: sv.bottomAnchor),
                content.widthAnchor.constraint(equalTo: sv.widthAnchor),
            ])
            return sv
        case 8, 46: // List, SectionList
            return UITableView(frame: .zero, style: .grouped)
        case 9: // Loading
            return UIActivityIndicatorView(style: .medium)
        case 20: // Icon
            let iv = UIImageView()
            iv.contentMode = .scaleAspectFit
            iv.tintColor = .label
            return iv
        case 21: // Avatar
            let iv = UIImageView()
            iv.contentMode = .scaleAspectFill
            iv.layer.cornerRadius = 24
            iv.clipsToBounds = true
            return iv
        case 22: // Badge
            let container = UIView()
            container.accessibilityIdentifier = "flash-badge"
            let label = UILabel()
            label.font = .systemFont(ofSize: 11, weight: .bold)
            label.textColor = .white
            label.backgroundColor = .systemRed
            label.textAlignment = .center
            label.layer.cornerRadius = 8
            label.clipsToBounds = true
            container.addSubview(label)
            return container
        case 23: // Divider
            let line = UIView()
            line.backgroundColor = .separator
            return line
        case 26: // ProgressBar
            return UIProgressView(progressViewStyle: .default)
        case 31: // Switch
            let sw = UISwitch()
            sw.addTarget(self, action: #selector(switchChanged(_:)), for: .valueChanged)
            return sw
        case 32: // Checkbox
            let btn = UIButton(type: .system)
            btn.accessibilityIdentifier = "flash-checkbox"
            btn.setImage(UIImage(systemName: "square"), for: .normal)
            btn.setImage(UIImage(systemName: "checkmark.square.fill"), for: .selected)
            btn.addTarget(self, action: #selector(buttonTapped(_:)), for: .touchUpInside)
            return btn
        case 51: // AppBar
            let bar = UIStackView()
            bar.axis = .horizontal
            bar.alignment = .center
            bar.spacing = 8
            bar.backgroundColor = .secondarySystemBackground
            bar.isLayoutMarginsRelativeArrangement = true
            bar.layoutMargins = UIEdgeInsets(top: 8, left: 16, bottom: 8, right: 16)
            return bar
        case 52: // TabBar
            let tabs = UIStackView()
            tabs.axis = .horizontal
            tabs.distribution = .fillEqually
            tabs.backgroundColor = .secondarySystemBackground
            return tabs
        case 54, 55, 69: // Modal, Sheet, BottomSheet
            let sheet = UIView()
            sheet.backgroundColor = UIColor.systemBackground.withAlphaComponent(0.98)
            sheet.layer.cornerRadius = 12
            sheet.layer.maskedCorners = [.layerMinXMinYCorner, .layerMaxXMinYCorner]
            return sheet
        default:
            return UIView()
        }
    }

    private func insert(_ child: UIView, into parent: UIView, at index: Int) {
        if let stack = parent as? UIStackView {
            let idx = min(index, stack.arrangedSubviews.count)
            stack.insertArrangedSubview(child, at: idx)
            return
        }
        if let scroll = parent as? UIScrollView,
           let stack = scroll.subviews.first as? UIStackView {
            let idx = min(index, stack.arrangedSubviews.count)
            stack.insertArrangedSubview(child, at: idx)
            return
        }
        parent.addSubview(child)
    }

    @objc private func buttonTapped(_ sender: UIButton) {
        if sender.accessibilityIdentifier == "flash-checkbox" {
            sender.isSelected.toggle()
        }
        fireHandler(for: sender)
    }

    @objc private func switchChanged(_ sender: UISwitch) {
        fireHandler(for: sender)
    }

    private func fireHandler(for sender: UIView) {
        guard let handle = views.first(where: { $0.value === sender })?.key,
              let handlerId = handlers[handle] else { return }
        flash_fire_handler(handlerId)
    }
}

// MARK: - C ABI (implemented in Rust static lib)

@_silgen_name("flash_host_register")
func flash_host_register(_ vtable: FlashHostVTable)

@_silgen_name("flash_host_apply_ops")
func flash_host_apply_ops(_ ptr: UnsafePointer<UInt8>?, _ len: UInt32)

@_silgen_name("flash_fire_handler")
func flash_fire_handler(_ handlerId: UInt32)

struct FlashHostVTable {
    var create: (@convention(c) (UInt32, UInt16) -> Void)?
    var set_prop: (@convention(c) (UInt32, UInt16, UnsafePointer<UInt8>?, UInt32) -> Void)?
    var insert_child: (@convention(c) (UInt32, UInt32, UInt32) -> Void)?
    var remove: (@convention(c) (UInt32) -> Void)?
    var set_handler: (@convention(c) (UInt32, UInt32) -> Void)?
    var set_frame: (@convention(c) (UInt32, Float, Float, Float, Float) -> Void)?
    var commit: (@convention(c) () -> Void)?
}
