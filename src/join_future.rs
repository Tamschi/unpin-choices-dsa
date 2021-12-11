//! A [`Future`] that can interlace [`Future`]s. Not threading!

use bitvec::prelude::*;
use core::{
	future::Future,
	mem::MaybeUninit,
	pin::Pin,
	task::{Context, Poll},
};
use pin_project::pin_project;
use project_uninit::partial_init;

pub fn join<Fs: Futures>(futures: Fs) -> JoinFuture<Fs> {
	JoinFuture::new(futures)
}

#[pin_project]
#[derive(Debug)]
pub struct JoinFuture<Fs: Futures> {
	completion: Fs::Completion,
	#[pin]
	futures: Fs,
	outputs: MaybeUninit<Fs::Outputs>,
}

impl<Fs: Futures> JoinFuture<Fs> {
	/// Creates a new instance of [`JoinFuture`] from the given `futures`.
	#[must_use]
	pub fn new(futures: Fs) -> Self {
		Self {
			completion: Fs::initial_completion(),
			futures,
			outputs: MaybeUninit::uninit(),
		}
	}
}

impl<Fs: Futures> Future for JoinFuture<Fs> {
	type Output = Fs::Outputs;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.project();

		match Fs::poll(this.completion, this.futures, this.outputs, cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(()) => Poll::Ready(unsafe {
				//SAFETY: Validity of this operation is directly required by [`JoinFutureFutures`]'s implementation contract.
				this.outputs.as_ptr().read()
			}),
		}
	}
}

/// # Safety
///
/// When [`JoinFutureFutures::poll`] returns [`Poll::Ready`],
/// then it must be valid to read `outputs` as initialised value once directly afterwards.
pub unsafe trait Futures: Sized {
	/// The combined output type.
	type Outputs;
	/// A way to track [`Future`] completion.
	type Completion;

	/// The initial `Self::Completion` value.
	fn initial_completion() -> Self::Completion;

	/// Like [`Future::poll`].
	fn poll(
		completion: &mut Self::Completion,
		futures: Pin<&mut Self>,
		outputs: &mut MaybeUninit<Self::Outputs>,
		cx: &mut Context<'_>,
	) -> Poll<()>;
}

unsafe impl Futures for () {
	type Outputs = ();
	type Completion = BitArr!(for 0);

	fn initial_completion() -> Self::Completion {
		BitArray::zeroed()
	}

	fn poll(
		_completion: &mut Self::Completion,
		_futures: Pin<&mut Self>,
		_results: &mut MaybeUninit<Self::Outputs>,
		_cx: &mut Context<'_>,
	) -> Poll<()> {
		Poll::Ready(())
	}
}

unsafe impl<F1> Futures for (F1,)
where
	F1: Future,
{
	type Outputs = (F1::Output,);
	type Completion = BitArr!(for 1);

	fn initial_completion() -> Self::Completion {
		BitArray::zeroed()
	}

	fn poll(
		completion: &mut Self::Completion,
		futures: Pin<&mut Self>,
		outputs: &mut MaybeUninit<Self::Outputs>,
		cx: &mut Context<'_>,
	) -> Poll<()> {
		let mut stepped = false;
		let mut incomplete = false;

		{
			let mut completion = completion.get_mut(0).unwrap();
			if !*completion {
				stepped = true;
				match unsafe { futures.map_unchecked_mut(|futures| &mut futures.0) }.poll(cx) {
					Poll::Pending => incomplete = true,
					Poll::Ready(output) => {
						partial_init!(outputs => 0 = output);
						*completion = true
					}
				}
			}
		}

		if incomplete {
			Poll::Pending
		} else {
			assert!(stepped, "`JoinFuture` was previously completed.");
			Poll::Ready(())
		}
	}
}
