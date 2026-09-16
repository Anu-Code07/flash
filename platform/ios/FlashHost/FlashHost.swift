// Flash iOS native renderer — UIKit mapping for 10 core widgets.
// Wire: Rust static lib calls flash_host_register() at launch.

import UIKit

// MARK: - Widget kinds (stable IDs from stl/flash-stl/src/widgets/core.rs)

enum FlashWidgetKind: UInt16 {
    case text = 0
    case button = 1
    case column = 2
    case row = 3
    case stack = 4
    case image = 5
    case textField = 6
    case scrollView = 7
    case list = 8
    case loading = 9
}

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
        case .title:
            if let button = view as? UIButton { button.setTitle(text, for: .normal) }
        case .value:
            if let field = view as? UITextField { field.text = text }
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
        switch FlashWidgetKind(rawValue: kind) {
        case .text:
            let label = UILabel()
            label.numberOfLines = 0
            return label
        case .button:
            let button = UIButton(type: .system)
            button.addTarget(self, action: #selector(buttonTapped(_:)), for: .touchUpInside)
            return button
        case .column:
            let stack = UIStackView()
            stack.axis = .vertical
            stack.alignment = .fill
            stack.spacing = 8
            return stack
        case .row:
            let stack = UIStackView()
            stack.axis = .horizontal
            stack.alignment = .center
            stack.spacing = 8
            return stack
        case .stack:
            return UIView()
        case .image:
            let iv = UIImageView()
            iv.contentMode = .scaleAspectFit
            iv.clipsToBounds = true
            return iv
        case .textField:
            return UITextField()
        case .scrollView:
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
        case .list:
            return UICollectionView(frame: .zero, collectionViewLayout: UICollectionViewFlowLayout())
        case .loading:
            return UIActivityIndicatorView(style: .medium)
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
