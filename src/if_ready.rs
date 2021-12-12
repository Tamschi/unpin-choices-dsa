//! A [`Future`] adaptor that is always ready, but may return [`None`].

use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

/// Returns a [`Future`] that is ready immediately,
/// returning [`None`] iff `future` is not ready immediately when polled.
pub fn if_ready<F>(future: F) -> IfReady<F> {
	IfReady(Some(future))
}

/// A [`Future`] that is always ready, but may return [`None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfReady<F>(Option<F>);

impl<F> Unpin for IfReady<F> {}

impl<F: Future> Future for IfReady<F> {
	type Output = Option<F::Output>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		unsafe {
			let output = match Pin::map_unchecked_mut(self.as_mut(), |this| &mut this.0)
				.as_pin_mut()

				// We could alternatively just return [`None`],
				// but that would make the API more error-prone to consume.
				.expect("`IfReady` erroneously polled twice")

				// Alternatively, we could pass a fake context here that just doesn't schedule anything.
				// We'd still have to drop the inner future before returning regardless, though.
				.poll(cx)
			{
				Poll::Pending => None,
				Poll::Ready(output) => Some(output),
			};

			// We *have* to drop the inner [`Future`] now, since we can't guarantee it won't stay in place.
			Pin::get_unchecked_mut(self).0 = None;
			Poll::Ready(output)
		}
	}
}
