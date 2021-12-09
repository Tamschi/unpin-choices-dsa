//! A minimal always-[`Unpin`] wrapper.

use core::{
	borrow::{Borrow, BorrowMut},
	mem,
	ops::{Deref, DerefMut},
	pin::Pin,
};

use alloc::{boxed::Box, rc::Rc, sync::Arc};

/// A minimal wrapper that is always [`Unpin`].
///
/// # Safety notes
///
/// `T` and [`AntiPinned<T>`] are interchangeable unless pinned, or whenever `T: Unpin`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AntiPinned<T: ?Sized>(pub T);

// `Unpin` is safe to implement as exposing `Self::0` would require `unsafe` already.
impl<T: ?Sized> Unpin for AntiPinned<T> {}

/// This wrapper is supposed to only change pinned behaviour,
/// so its contents are readily accessible.
impl<T: ?Sized> Deref for AntiPinned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T: ?Sized> DerefMut for AntiPinned<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T: ?Sized> AntiPinned<T> {
	/// Gives plain exclusive access to the unpinned value
	/// inside this pinned [`AntiPinned<T>`].
	#[must_use]
	pub fn as_unpinned(self: Pin<&mut Self>) -> &mut T {
		// This can be implemented using safe Rust because `AntiPin<_>` is `Unpin`.
		&mut Pin::into_inner(self).0
	}
}

impl<T: ?Sized> Borrow<T> for AntiPinned<T> {
	fn borrow(&self) -> &T {
		&self.0
	}
}
impl<T: ?Sized> BorrowMut<T> for AntiPinned<T> {
	fn borrow_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<T> From<T> for AntiPinned<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

/// Implementing this for [`Box`] is specifically allowed.
impl<T: ?Sized> From<Box<T>> for Box<AntiPinned<T>> {
	fn from(boxed: Box<T>) -> Self {
		AntiPinned::wrap_boxed(boxed)
	}
}

macro_rules! boxed_conversions {
	// There's a bit of syntax noise here. In short:
	// `{$(…),*$(,)?} => {$(…)}` cleanly makes a macro variadic over a comma-separated pattern.
	//
	// The curly brackets above are interchangeable with round ones.
	// I like to use those that reflect how I expect the macro to be used.
	{$(
		$box:ident($wrap:ident, $unwrap:ident, $wrap_pinned:ident$(, $unwrap_pinned:ident)?$(,)?)
	),*$(,)?} => {$(
		impl<T: ?Sized> AntiPinned<T> {
			/// Wraps a boxed value in [`AntiPinned<_>`], in place.
			#[must_use]
			pub fn $wrap(boxed: $box<T>) -> Box<Self> {
				//SAFETY: See <`AntiPinned`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

			/// Unwraps a boxed [`AntiPinned<_>`] in place.
			#[must_use]
			pub fn $unwrap(boxed: $box<Self>) -> Box<T> {
				//SAFETY: See <`AntiPinned`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

			/// Wraps a pinned boxed value in [`AntiPinned<_>`], in place.
			#[must_use]
			pub fn $wrap_pinned(boxed: $box<T>) -> Box<Self>
			where
				T: Unpin
			{
				//SAFETY: See <`AntiPinned`#safety-notes>.
				unsafe { mem::transmute(boxed) }
			}

			// This part is optional.
			$(
				/// Unwraps a pinned boxed [`AntiPinned<_>`] in place.
				#[must_use]
				pub fn $unwrap_pinned(boxed: $box<Self>) -> Box<T>
				where
					T: Unpin
				{
					//SAFETY: See <`AntiPinned`#safety-notes>.
					unsafe { mem::transmute(boxed) }
				}
			)?
		}
	)*};
}

boxed_conversions! {
	Box(wrap_boxed, unwrap_boxed, wrap_pinned_boxed),
	Rc(wrap_rced, unwrap_rced, wrap_pinned_rced, unwrap_pinned_rced),
	Arc(wrap_arced, unwrap_arced, wrap_pinned_arced, unwrap_pinned_arced),
}

/// Behind the exclusively-owning [`Box<T>`] alone,
/// we don't need `where T: Unpin` to unwrap the instance for free.
///
/// Instead, the instance of `T` may become irreversibly pinned in the process.
impl<T: ?Sized> AntiPinned<T> {
	/// Unwraps a pinned boxed [`AntiPinned<_>`] in place.
	#[must_use]
	pub fn unwrap_pinned_boxed(boxed: Box<Self>) -> Box<T> {
		//SAFETY: See <`AntiPinned`#safety-notes>.
		unsafe { mem::transmute(boxed) }
	}
}
