//! A small knockoff subset of [`tap`] that works with [`unsafe fn`](https://doc.rust-lang.org/stable/std/primitive.fn.html)
//! callbacks.

/// [`tap::Pipe`] but for [`unsafe fn`](https://doc.rust-lang.org/stable/std/primitive.fn.html)
/// callbacks.
pub trait PipeUnchecked {
	/// Pipes `self` into `func`.
	///
	/// # Safety
	///
	/// See `func`.
	unsafe fn pipe_unchecked<R>(self, func: unsafe fn(Self) -> R) -> R
	where
		Self: Sized,
		R: Sized,
	{
		func(self)
	}
}

impl<T: ?Sized> PipeUnchecked for T {}

/// `.map` but for [`unsafe fn`](https://doc.rust-lang.org/stable/std/primitive.fn.html)
/// callbacks.
pub trait MapUnchecked<T> {
	/// Maps `self` using `func`.
	///
	/// # Safety
	///
	/// See `func`.
	unsafe fn map_unchecked<R>(self, func: unsafe fn(T) -> R) -> Option<R>
	where
		Self: Sized,
		T: Sized,
		R: Sized;
}

impl<T> MapUnchecked<T> for Option<T> {
	unsafe fn map_unchecked<R>(self, func: unsafe fn(T) -> R) -> Option<R>
	where
		Self: Sized,
		T: Sized,
		R: Sized,
	{
		self.map(|value| func(value))
	}
}
