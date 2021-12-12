//! A [`Future`] adaptor that is either immediately ready or never completes.

use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

/// Returns a [`Future`] that is either ready immediately or,
/// iff `future` is not ready when polled, will never complete at all.
pub fn ready_or_never<F>(future: F) -> ReadyOrNever<F> {
	ReadyOrNever(Some(future))
}

/// A [`Future`] that is always ready, but may return [`None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadyOrNever<F>(Option<F>);

impl<F> Unpin for ReadyOrNever<F> {}

impl<F: Future> Future for ReadyOrNever<F> {
	type Output = F::Output;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		unsafe {
			let output = Pin::map_unchecked_mut(self.as_mut(), |this| &mut this.0)
				.as_pin_mut()
				// Alternatively, we could pass a fake context here that just doesn't schedule anything.
				// We'd still have to drop the inner future before returning regardless, though.
				.map_or(Poll::Pending, |inner| inner.poll(cx));
			// We *have* to drop the inner [`Future`] now, since we can't guarantee it won't stay in place.
			Pin::get_unchecked_mut(self).0 = None;
			output
		}
	}
}
