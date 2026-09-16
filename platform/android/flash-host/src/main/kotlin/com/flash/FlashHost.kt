package com.flash

import android.content.Context
import android.view.View
import android.view.ViewGroup
import android.widget.*
import androidx.recyclerview.widget.RecyclerView
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Flash Android native renderer — View system mapping for 10 core widgets.
 * Rust .so calls [applyOps] with encoded command buffer each frame.
 */
object FlashHost {
    private val views = mutableMapOf<Int, View>()
    private val handlers = mutableMapOf<Int, Int>()
    var rootView: ViewGroup? = null
    private lateinit var appContext: Context

    fun init(context: Context) {
        appContext = context.applicationContext
    }

    // ── Widget kinds (stable IDs from widgets/core.rs) ─────────────────────

    private const val TEXT = 0
    private const val BUTTON = 1
    private const val COLUMN = 2
    private const val ROW = 3
    private const val STACK = 4
    private const val IMAGE = 5
    private const val TEXT_FIELD = 6
    private const val SCROLL_VIEW = 7
    private const val LIST = 8
    private const val LOADING = 9

    private const val PROP_TEXT = 0
    private const val PROP_TITLE = 1
    private const val PROP_VALUE = 2
    private const val PROP_SRC = 3

    /** Apply encoded ops from Rust (one JNI crossing per frame). */
    @JvmStatic
    fun applyOps(bytes: ByteArray) {
        val buf = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
        while (buf.hasRemaining()) {
            when (buf.get().toInt() and 0xFF) {
                0 -> create(buf.int, buf.short.toInt() and 0xFFFF)
                1 -> setProp(buf.int, buf.short.toInt() and 0xFFFF, buf)
                2 -> insertChild(buf.int, buf.int, buf.int)
                3 -> remove(buf.int)
                4 -> {
                    val handle = buf.int
                    handlers[handle] = buf.int
                }
                5 -> setFrame(buf.int, buf.float, buf.float, buf.float, buf.float)
            }
        }
    }

    private fun setFrame(handle: Int, x: Float, y: Float, width: Float, height: Float) {
        views[handle]?.let { view ->
            val w = width.toInt().coerceAtLeast(1)
            val h = height.toInt().coerceAtLeast(1)
            view.layout(x.toInt(), y.toInt(), x.toInt() + w, y.toInt() + h)
        }
    }

    private fun create(handle: Int, kind: Int) {
        val view = makeView(kind)
        views[handle] = view
        if (rootView == null && view is LinearLayout) {
            rootView = view
        }
    }

    private fun setProp(handle: Int, key: Int, buf: ByteBuffer) {
        val view = views[handle] ?: return
        val text = decodeValueString(buf)
        when (key) {
            PROP_TEXT -> when (view) {
                is TextView -> view.text = text
                is EditText -> view.setText(text)
            }
            PROP_TITLE -> if (view is Button) view.text = text
            PROP_VALUE -> if (view is EditText) view.setText(text)
            PROP_SRC -> if (view is ImageView) {
                // URL loading via Coil/Glide in production apps
            }
        }
    }

    private fun insertChild(parent: Int, child: Int, index: Int) {
        val childView = views[child] ?: return
        val parentView = views[parent] as? ViewGroup ?: rootView ?: return
        val idx = index.coerceAtMost(parentView.childCount)
        if (childView.parent != null) {
            (childView.parent as ViewGroup).removeView(childView)
        }
        parentView.addView(childView, idx)
    }

    private fun remove(handle: Int) {
        views[handle]?.let { (it.parent as? ViewGroup)?.removeView(it) }
        views.remove(handle)
    }

    private fun makeView(kind: Int): View {
        return when (kind) {
            TEXT -> TextView(appContext)
            BUTTON -> Button(appContext).apply {
                setOnClickListener {
                    val handle = views.entries.find { it.value === this }?.key
                    if (handle != null) handlers[handle]?.let { id ->
                        nativeFireHandler(id)
                    }
                }
            }
            COLUMN -> LinearLayout(appContext).apply { orientation = LinearLayout.VERTICAL }
            ROW -> LinearLayout(appContext).apply { orientation = LinearLayout.HORIZONTAL }
            STACK -> FrameLayout(appContext)
            IMAGE -> ImageView(appContext)
            TEXT_FIELD -> EditText(appContext)
            SCROLL_VIEW -> ScrollView(appContext).apply {
                addView(LinearLayout(appContext).apply {
                    orientation = LinearLayout.VERTICAL
                })
            }
            LIST -> RecyclerView(appContext)
            LOADING -> ProgressBar(appContext)
            else -> View(appContext)
        }
    }

    private fun decodeValueString(buf: ByteBuffer): String {
        val tag = buf.get().toInt() and 0xFF
        if (tag != 0) return ""
        val len = buf.short.toInt() and 0xFFFF
        val bytes = ByteArray(len)
        buf.get(bytes)
        return String(bytes, Charsets.UTF_8)
    }

    @JvmStatic
    private external fun nativeFireHandler(handlerId: Int)
}
