use freya::prelude::*;


pub fn set_clickable(clickable: bool) {
  Platform::get().with_window(None, move |w| {
    let _ = w.set_cursor_hittest(clickable);

    // On X11/KDE, set_cursor_hittest is not enough for click-through
    #[cfg(target_os = "linux")]
    {
      use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
      use x11rb::{
        connection::Connection,
        protocol::{
          shape::{self, ConnectionExt as _},
          xproto::ClipOrdering,
        },
      };

      let Ok(handle) = w.window_handle() else {
        warn!("Failed to get window handle");
        return;
      };

      let xid: u32 = match handle.as_raw() {
        RawWindowHandle::Xcb(h) => h.window.get(),
        RawWindowHandle::Xlib(h) => h.window as u32,
        //  Wayland/etc. doesn't need this hack
        _ => return,
      };

      let Ok((conn, _)) = x11rb::connect(None) else {
        warn!("Failed to connect to X server");
        return;
      };

      let rects: &[x11rb::protocol::xproto::Rectangle] = if clickable {
        &[x11rb::protocol::xproto::Rectangle {
          x: 0,
          y: 0,
          width: u16::MAX,
          height: u16::MAX,
        }]
      } else {
        &[]
      };

      let _ = conn.shape_rectangles(
        shape::SO::SET,
        shape::SK::INPUT,
        ClipOrdering::UNSORTED,
        xid,
        0,
        0,
        rects,
      );
      let _ = conn.flush();
    }
  });
}
