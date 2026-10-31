use std::{cell::RefCell, rc::Rc};

use freya::prelude::*;

pub fn use_commit_on_blur(
  id: AccessibilityId,
  value: State<String>,
  commit: impl FnMut(String) + Clone + 'static,
) {
  let focus = use_focus(id);
  let committed = use_hook(|| Rc::new(RefCell::new(value.peek().clone())));

  use_side_effect({
    let committed = committed.clone();
    let mut commit = commit.clone();
    move || {
      if focus.read().is_focused() {
        return;
      }

      let current = value.read().clone();
      if current != *committed.borrow() {
        committed.replace(current.clone());
        commit(current);
      }
    }
  });

  let mut commit = commit;
  use_drop(move || {
    let Some(current) = value.try_peek().map(|current| current.clone()) else {
      return;
    };

    if current != *committed.borrow() {
      commit(current);
    }
  });
}
