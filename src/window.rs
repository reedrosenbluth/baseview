use std::marker::PhantomData;

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use crate::event::{Event, EventStatus};
use crate::window_open_options::WindowOpenOptions;
use crate::{MouseCursor, Point, Size};

#[cfg(target_os = "macos")]
use crate::macos as platform;
#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(target_os = "linux")]
use crate::x11 as platform;

pub struct WindowHandle {
    window_handle: platform::WindowHandle,
    // so that WindowHandle is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl WindowHandle {
    fn new(window_handle: platform::WindowHandle) -> Self {
        Self { window_handle, phantom: PhantomData }
    }

    /// Close the window
    pub fn close(&mut self) {
        self.window_handle.close();
    }

    /// Returns `true` if the window is still open, and returns `false`
    /// if the window was closed/dropped.
    pub fn is_open(&self) -> bool {
        self.window_handle.is_open()
    }
}

unsafe impl HasRawWindowHandle for WindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_handle.raw_window_handle()
    }
}

pub trait WindowHandler {
    fn on_frame(&mut self, window: &mut Window);
    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus;
}

pub struct Window<'a> {
    window: platform::Window<'a>,

    // so that Window is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl<'a> Window<'a> {
    #[cfg(target_os = "windows")]
    pub(crate) fn new(window: platform::Window<'a>) -> Window<'a> {
        Window { window, phantom: PhantomData }
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn new(window: platform::Window) -> Window {
        Window { window, phantom: PhantomData }
    }

    pub fn open_parented<P, H, B>(parent: &P, options: WindowOpenOptions, build: B) -> WindowHandle
    where
        P: HasRawWindowHandle,
        H: WindowHandler + 'static,
        B: FnOnce(&mut Window) -> H,
        B: Send + 'static,
    {
        let window_handle = platform::Window::open_parented::<P, H, B>(parent, options, build);
        WindowHandle::new(window_handle)
    }

    pub fn open_blocking<H, B>(options: WindowOpenOptions, build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut Window) -> H,
        B: Send + 'static,
    {
        platform::Window::open_blocking::<H, B>(options, build)
    }

    /// Close the window
    pub fn close(&mut self) {
        self.window.close();
    }

    /// Resize the window to the given size. The size is always in logical pixels. DPI scaling will
    /// automatically be accounted for.
    pub fn resize(&mut self, size: Size) {
        self.window.resize(size);
    }

    pub fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.window.set_mouse_cursor(cursor);
    }

    pub fn has_focus(&mut self) -> bool {
        self.window.has_focus()
    }

    pub fn focus(&mut self) {
        self.window.focus()
    }

    /// Enable or disable unbounded mouse movement mode for drag operations.
    ///
    /// When enabled:
    /// - The cursor is hidden
    /// - The cursor position is frozen (on screen)
    /// - Mouse movement events report relative deltas from the starting position
    ///
    /// When disabled:
    /// - If `restore_position` was true when enabled, the cursor returns to its original position
    /// - The cursor becomes visible again
    ///
    /// This is useful for implementing drag operations (like knobs/sliders) where you want
    /// unlimited drag range without the cursor hitting screen edges.
    pub fn enable_unbounded_mouse_movement(&mut self, enable: bool, restore_position: bool) {
        self.window.enable_unbounded_mouse_movement(enable, restore_position);
    }

    /// Check if unbounded mouse movement is currently enabled.
    pub fn is_unbounded_mouse_movement_enabled(&self) -> bool {
        self.window.is_unbounded_mouse_movement_enabled()
    }

    /// Set the cursor visibility.
    ///
    /// Note: This is reference-counted internally on some platforms (macOS).
    /// Each call to hide should be matched with a call to show.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    /// Set the cursor position in logical coordinates relative to the window.
    ///
    /// Returns `Ok(())` on success, `Err(())` if the operation failed.
    pub fn set_cursor_position(&mut self, position: Point) -> Result<(), ()> {
        self.window.set_cursor_position(position)
    }

    /// If provided, then an OpenGL context will be created for this window. You'll be able to
    /// access this context through [crate::Window::gl_context].
    #[cfg(feature = "opengl")]
    pub fn gl_context(&self) -> Option<&crate::gl::GlContext> {
        self.window.gl_context()
    }
}

unsafe impl<'a> HasRawWindowHandle for Window<'a> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}

unsafe impl<'a> HasRawDisplayHandle for Window<'a> {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}
