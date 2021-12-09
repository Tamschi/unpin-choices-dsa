//! A minimal content-pinning wrapper.

use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::{
	borrow::{Borrow, BorrowMut},
	mem,
	ops::{Deref, DerefMut},
	pin::Pin,
};

/// A minimal wrapper that can pin its contents.
///
/// # Safety notes
///
/// `T` and [`PinnedPin<T>`] are interchangeable pinned and unpinned each,
/// but not necessarily between pinned and unpinned state.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PinnedPin<T: ?Sized>(pub T);

/// This is implicit and only for illustration purposes, so I deactivate it here with `cfg(FALSE)`.
#[cfg(FALSE)]
impl<T: ?Sized> Unpin for PinnedPin<T> where T: Unpin {}

/// Again a very transparent wrapper.
/// [`Deref`] and [`DerefMut`] are largely irrelevant for pinning.
impl<T: ?Sized> Deref for PinnedPin<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T: ?Sized> DerefMut for PinnedPin<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// The shared and exclusive versions must be implemented separately,
/// as [`Deref`]'s implementation isn't a "pin projection", i.e. an accessor involving [`Pin<_>`],
/// and no such projection is made available automatically.
///
/// # Safety notes
///
/// [`PinnedPin`] can expose its field only as pinned because
/// it never gives access to `&mut T` while it is pinned itself.
impl<T: ?Sized> PinnedPin<T> {
	/// Gives pinning shared access to the pinned value
	/// inside this pinned [`PinnedPin<T>`].
	#[must_use]
	pub fn as_pinned(self: Pin<&Self>) -> Pin<&T> {
		unsafe { self.map_unchecked(Self::deref) }
	}

	/// Gives pinning exclusive access to the pinned value
	/// inside this pinned [`PinnedPin<T>`].
	#[must_use]
	pub fn as_mut_pinned(self: Pin<&mut Self>) -> Pin<&mut T> {
		unsafe { self.map_unchecked_mut(Self::deref_mut) }
	}
}

impl<T: ?Sized> Borrow<T> for PinnedPin<T> {
	fn borrow(&self) -> &T {
		&self.0
	}
}
impl<T: ?Sized> BorrowMut<T> for PinnedPin<T> {
	fn borrow_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<T> From<T> for PinnedPin<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

/// Implementing this for [`Box`] is specifically allowed.
impl<T: ?Sized> From<Box<T>> for Box<PinnedPin<T>> {
	fn from(boxed: Box<T>) -> Self {
		PinnedPin::wrap_boxed(boxed)
	}
}

impl<T> From<Pin<Box<T>>> for Pin<Box<PinnedPin<T>>> {
	fn from(boxed: Pin<Box<T>>) -> Self {
		PinnedPin::wrap_pinned_boxed(boxed)
	}
}

macro_rules! boxed_conversions {
	// There's a bit of syntax noise here. In short:
	// `{$(…),*$(,)?} => {$(…)}` cleanly makes a macro variadic over a comma-separated pattern.
	//
	// The curly brackets above are interchangeable with round ones.
	// I like to use those that reflect how I expect the macro to be used.
	{$(
		$box:ident($wrap:ident, $unwrap:ident, $wrap_pinned:ident, $unwrap_pinned:ident$(,)?)
	),*$(,)?} => {$(
		impl<T: ?Sized> PinnedPin<T> {
			/// Wraps a boxed value in [`PinnedPin<_>`], in place.
			#[must_use]
			pub fn $wrap(boxed: $box<T>) -> Box<Self> {
				//SAFETY: See <`PinnedPin`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

			/// Unwraps a boxed [`PinnedPin<_>`] in place.
			#[must_use]
			pub fn $unwrap(boxed: $box<Self>) -> Box<T> {
				//SAFETY: See <`PinnedPin`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

			/// Wraps a pinned boxed value in [`PinnedPin<_>`], in place.
			#[must_use]
			pub fn $wrap_pinned(boxed: Pin<$box<T>>) -> Pin<Box<Self>> {
				//SAFETY: See <`PinnedPin`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

				/// Unwraps a pinned boxed [`PinnedPin<_>`] in place.
				#[must_use]
				pub fn $unwrap_pinned(boxed: Pin<$box<Self>>) -> Pin<Box<T>> {
					//SAFETY: See <`PinnedPin`#safety-notes>.
					unsafe { mem::transmute(boxed) }
				}
		}
	)*};
}

boxed_conversions! {
	Box(wrap_boxed, unwrap_boxed, wrap_pinned_boxed, unwrap_pinned_boxed),
	Rc(wrap_rced, unwrap_rced, wrap_pinned_rced, unwrap_pinned_rced),
	Arc(wrap_arced, unwrap_arced, wrap_pinned_arced, unwrap_pinned_arced),
}

/// It's also possible to reinterpret references, *even pinned ones*.
///
/// Note that the other direction, to `&T`, `&mut T`, `Pin<&T>` and `Pin<&mut T>`,
/// is already covered through [`Deref`], [`DerefMut`], [`.as_pinned(…)`](PinnedPin::as_pinned) and [`.as_mut_pinned(…)`]((PinnedPin::as_mut_pinned)).
impl<T: ?Sized> PinnedPin<T> {
	/// Reinterprets a reference so that the target is wrapped in [`PinnedPin<_>`].
	#[must_use]
	pub fn from_ref(reference: &T) -> &Self {
		unsafe {
			//SAFETY: This is a direct reinterpret-cast between the compatible `T` and `PinnedPin<T>`.
			&*(reference as *const _ as *const _)
		}
	}

	/// Reinterprets an exclusive reference so that the target is wrapped in [`PinnedPin<_>`].
	#[must_use]
	pub fn from_mut(reference: &mut T) -> &mut Self {
		unsafe {
			//SAFETY: This is a direct reinterpret-cast between the compatible `T` and `PinnedPin<T>`.
			&mut *(reference as *mut _ as *mut _)
		}
	}

	/// Reinterprets a reference so that the target is wrapped in [`PinnedPin<_>`].
	#[must_use]
	pub fn from_pin_ref(reference: Pin<&T>) -> Pin<&Self> {
		unsafe {
			//SAFETY: This is a direct reinterpret-cast between the compatible `T` and `PinnedPin<T>`.
			mem::transmute(reference)
		}
	}

	/// Reinterprets an exclusive reference so that the target is wrapped in [`PinnedPin<_>`].
	#[must_use]
	pub fn from_pin_mut(reference: Pin<&mut T>) -> Pin<&mut Self> {
		unsafe {
			//SAFETY: This is a direct reinterpret-cast between the compatible `T` and `PinnedPin<T>`.
			mem::transmute(reference)
		}
	}
}
