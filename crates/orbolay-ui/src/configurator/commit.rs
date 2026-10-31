use std::{cell::RefCell, rc::Rc};

use freya::prelude::*;

pub fn use_commit_on_blur(
  id: AccessibilityId,
  value: State<String>,
  mut commit: impl FnMut(String) + Clone + 'static,
) {
  let focus = use_focus(id);
  let committed = use_hook(|| Rc::new(RefCell::new(value.peek().clone())));

  let mut flush = move |current: String| {
    if current != *committed.borrow() {
      committed.replace(current.clone());
      commit(current);
    }
  };

  use_side_effect({
    let mut flush = flush.clone();
    move || {
      if !focus.read().is_focused() {
        let current = value.read().clone();
        flush(current);
      }
    }
  });

  use_drop(move || {
    let current = value.try_peek().map(|current| current.clone());
    if let Some(current) = current {
      flush(current);
    }
  });
}
