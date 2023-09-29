use crate::recursion_guard::{RecursionState, RecursionToken};

use super::Extra;

pub struct ValidationState<'a> {
    pub recursion_depth: u16,
    // deliberately make Extra readonly
    extra: Extra<'a>,
    #[cfg(debug_assertions)]
    initial_recursion_depth: u16,
}

impl<'a> ValidationState<'a> {
    pub fn new(extra: Extra<'a>, recursion_depth: u16) -> Self {
        Self {
            recursion_depth,
            extra,
            #[cfg(debug_assertions)]
            initial_recursion_depth: recursion_depth,
        }
    }

    pub fn with_new_extra<'r, R: 'r>(
        &mut self,
        extra: Extra<'_>,
        f: impl for<'s> FnOnce(&'s mut ValidationState<'_>) -> R,
    ) -> R {
        // TODO: It would be nice to implement this function with a drop guard instead of a closure,
        // but lifetimes get in a tangle. Maybe someone brave wants to have a go at unpicking lifetimes.
        let mut new_state = ValidationState {
            recursion_depth: self.recursion_depth,
            extra,
            #[cfg(debug_assertions)]
            initial_recursion_depth: self.recursion_depth,
        };
        f(&mut new_state)
    }

    /// Temporarily rebinds the extra field by calling `f` to modify extra.
    ///
    /// When `ValidationStateWithReboundExtra` drops, the extra field is restored to its original value.
    pub fn rebind_extra<'state>(
        &'state mut self,
        f: impl FnOnce(&mut Extra<'a>),
    ) -> ValidationStateWithReboundExtra<'state, 'a> {
        #[allow(clippy::unnecessary_struct_initialization)]
        let old_extra = Extra { ..self.extra };
        f(&mut self.extra);
        ValidationStateWithReboundExtra { state: self, old_extra }
    }

    pub fn extra(&self) -> &'_ Extra<'a> {
        &self.extra
    }

    pub fn strict_or(&self, default: bool) -> bool {
        self.extra.strict.unwrap_or(default)
    }

    pub fn descend<'token, 's>(&'s mut self) -> RecursionToken<'token, ValidationState<'a>>
    where
        's: 'token,
    {
        RecursionToken::new(self)
    }
}

impl<'a> RecursionState for ValidationState<'a> {
    fn incr_depth(&mut self) -> u16 {
        self.recursion_depth += 1;
        self.recursion_depth
    }

    fn decr_depth(&mut self) {
        self.recursion_depth -= 1;
    }
}

#[cfg(debug_assertions)]
// This is just a sanity check that runs in our CI to catch any mistakes
// in the usage of ValidationState or RecursionToken.
impl Drop for ValidationState<'_> {
    fn drop(&mut self) {
        assert!(
            self.recursion_depth == self.initial_recursion_depth,
            "ValidationState dropped with non-zero recursion depth of {}",
            self.recursion_depth - self.initial_recursion_depth
        );
    }
}

pub struct ValidationStateWithReboundExtra<'state, 'a> {
    state: &'state mut ValidationState<'a>,
    old_extra: Extra<'a>,
}

impl<'a> std::ops::Deref for ValidationStateWithReboundExtra<'_, 'a> {
    type Target = ValidationState<'a>;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'a> std::ops::DerefMut for ValidationStateWithReboundExtra<'_, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl Drop for ValidationStateWithReboundExtra<'_, '_> {
    fn drop(&mut self) {
        std::mem::swap(&mut self.state.extra, &mut self.old_extra);
    }
}
