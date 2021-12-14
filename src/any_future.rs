//! A [`Future`] that interlaces [`Future`]s, until one completes.

use crate::pinned_pin::PinnedPin;
use alloc::boxed::Box;
use core::{
	future::Future,
	mem,
	pin::Pin,
	task::{Context, Poll},
};
use pin_project::pin_project;
use tap::Pipe;

/// Creates a [`Future`] that completes when any [`Future`] in `futures` completes.
///
/// The output is the output of that [`Future`].
pub fn any<Fs: Futures>(futures: Fs) -> AnyFuture<Fs> {
	AnyFuture::new(futures)
}

/// A [`Future`] that completes when any [`Future`] in `futures` completes.
///
/// Note that this type can't implement [`futures_core::FusedFuture`] without storing an additional completion flag,
/// at which point composing that externally only when needed is generally better.
///
/// > It's pretty neat that we can do this also without a macro,
/// > since that *may* lead to lower compile times due to less total emitted code.
/// >
/// > It's not as versatile as a macro if we don't control storage for the composed futures, though.
///
/// Compare and contrast [`crate::join_future::JoinFuture`].
#[derive(Debug)]
#[pin_project]
#[repr(transparent)]
pub struct AnyFuture<Fs: Futures + ?Sized> {
	/// We can actually implement this entire type in safe Rust (except for one constructor),
	/// by using the item-pinning [`PinnedPin`] here.
	#[pin]
	futures: PinnedPin<Fs>,
}

impl<Fs: Futures + ?Sized> AnyFuture<Fs> {
	/// Creates a new instance of [`AnyFuture`] from the given `futures`.
	#[must_use]
	pub fn new(futures: Fs) -> Self
	where
		Fs: Sized,
	{
		Self {
			futures: futures.into(),
		}
	}

	/// Creates a new instance of [`AnyFuture`] from the given `futures`.
	#[must_use]
	pub fn new_boxed(futures: Box<Fs>) -> Box<Self> {
		// *Technically* it's legal to transmute directly here
		// (and that's what this code should do when compiler-optimised)
		// but I prefer to let the compiler check what's documented elsewhere, if possible.
		let futures: Box<PinnedPin<Fs>> = futures.into();
		unsafe {
			//SAFETY:`AnyFuture` and `PinnedPin` have the same memory representation.
			mem::transmute(futures)
		}
	}

	/// Creates a new instance of [`AnyFuture`] from the given `futures`.
	#[must_use]
	pub fn new_pinned(futures: Pin<Box<Fs>>) -> Pin<Box<Self>> {
		unsafe {
			//SAFETY:
			// This is sound because `Self` is dependently `!Unpin` on `Fs`
			// (through `PinnedPin`) and …
			Pin::into_inner_unchecked(futures)
				.pipe(Self::new_boxed)
				// because we "repin" the result here:
				// (Technically the data remained pinned of course,
				// so we just fix up the return type.)
				.into()
		}
	}
}

impl<Fs: Futures + Sized> Future for AnyFuture<Fs> {
	type Output = Fs::Output;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		Fs::poll(self.project().futures, cx)
	}
}

/// Types that can be used with [`AnyFuture`].
///
/// Compare and contrast [`crate::join_future::futures`].
pub trait Futures {
	/// The combined output type.
	type Output;

	/// Like [`Future::poll`], with [`PinnedPin`] just for convenience.
	///
	/// This *should* use `self`, but that's not here supported by Rust (as of Rust 1.57).
	fn poll(futures: Pin<&mut PinnedPin<Self>>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

impl<F: Future, const N: usize> Futures for [F; N] {
	type Output = F::Output;

	fn poll(futures: Pin<&mut PinnedPin<Self>>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		for future in futures {
			if let ready @ Poll::Ready(_) = future.poll(cx) {
				return ready;
			}
		}
		Poll::Pending
	}
}

/// The implementation for slices is the same as for arrays,
/// but the compiled result is different:
///
/// The `.poll` method won't be monomorphised for each slice length (so the output text size is smaller),
/// but in exchange the loop can't be unrolled (as well), which means this will run slightly slower in SOME cases.
impl<F: Future> Futures for [F] {
	type Output = F::Output;

	fn poll(futures: Pin<&mut PinnedPin<Self>>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		for future in futures {
			if let ready @ Poll::Ready(_) = future.poll(cx) {
				return ready;
			}
		}
		Poll::Pending
	}
}

impl Futures for () {
	/// This *should* be `!`, but the never type isn't stable yet as of Rust 1.57.
	type Output = core::convert::Infallible;

	fn poll(_: Pin<&mut PinnedPin<Self>>, _: &mut Context<'_>) -> Poll<Self::Output> {
		Poll::Pending
	}
}

impl<F0: Future> Futures for (F0,) {
	type Output = F0::Output;

	fn poll(futures: Pin<&mut PinnedPin<Self>>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		//TODO: Implement that projection in [`PinnedPin`] instead.
		unsafe { futures.map_unchecked_mut(|this| &mut this.0 .0) }.poll(cx)
	}
}

impl<F0, F1> Futures for (F0, F1)
where
	F0: Future,
	F1: Future<Output = F0::Output>,
{
	type Output = F0::Output;

	fn poll(mut futures: Pin<&mut PinnedPin<Self>>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		//TODO: Implement that projection in [`PinnedPin`] instead.
		if let ready @ Poll::Ready(_) =
			unsafe { futures.as_mut().map_unchecked_mut(|this| &mut this.0 .0) }.poll(cx)
		{
			return ready;
		}
		if let ready @ Poll::Ready(_) =
			unsafe { futures.map_unchecked_mut(|this| &mut this.0 .1) }.poll(cx)
		{
			return ready;
		}
		Poll::Pending
	}
}

// etc.
