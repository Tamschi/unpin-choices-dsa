//! [`PinningPin`] can pin [array](https://doc.rust-lang.org/stable/core/primitive.array.html)
//! and [slice](https://doc.rust-lang.org/stable/core/primitive.slice.html)
//! items.
//!
//! This can't be implemented for arbitrary collections, since any collection with interior-mutability
//! could violate the pinning guarantees very easily even through shared references.
//!
//! The API of arrays and slices is unproblematic, however.
//!
//! # A note on indexing
//!
//! [`Index`](`core::ops::Index`) and [`IndexMut`](`core::ops::IndexMut`)
//! can't be implemented as it can't return a pinning reference.
//!
//! Instead, pinning accessors (`.get` etc.) are made available.
//!
//! > Personally I hope that their definitions will shift a bit once [GATs](https://github.com/rust-lang/rust/issues/44265)
//! > land to allow other kinds of borrowing or even by-value return types,
//! > but this would definitely take a breaking change and as such a new Rust edition.

use crate::{
	pinned_pin::PinnedPin,
	unchecked_tap::{MapUnchecked, PipeUnchecked},
};
use core::{
	iter::{FusedIterator, IntoIterator},
	pin::Pin,
};
use tap::Pipe;

impl<Item, const N: usize> PinnedPin<[Item; N]> {
	/// Views the pinned shared [array](https://doc.rust-lang.org/stable/core/primitive.array.html)
	/// as pinned shared [slice](https://doc.rust-lang.org/stable/core/primitive.slice.html).
	#[must_use]
	pub fn as_pinned_slice(self: Pin<&Self>) -> Pin<&[Item]> {
		unsafe {
			Pin::into_inner_unchecked(self)
				.as_slice()
				.pipe_unchecked(Pin::new_unchecked)
		}
	}

	/// Views the pinned [array](https://doc.rust-lang.org/stable/core/primitive.array.html)
	/// as pinned [slice](https://doc.rust-lang.org/stable/core/primitive.slice.html),
	/// exclusively.
	#[must_use]
	pub fn as_pinned_mut_slice(self: Pin<&mut Self>) -> Pin<&mut [Item]> {
		unsafe {
			Pin::into_inner_unchecked(self)
				.as_mut_slice()
				.pipe_unchecked(Pin::new_unchecked)
		}
	}
}

/// The accessors are available only on slices, anyway.
///
/// This isn't the full set that could be implemented, but it's a good chunk.
impl<Item> PinnedPin<[Item]> {
	/// Retrieves a shared reference to a pinned item,
	/// or [`None`] exactly whenever `index >= N`.
	#[must_use]
	pub fn get(self: Pin<&Self>, index: usize) -> Option<Pin<&Item>> {
		unsafe {
			Pin::into_inner_unchecked(self)
					// This next line is automatically not recursive since the reference isn't pinning here anymore.
					//
					// It's resolved using [`Deref` coercion](https://doc.rust-lang.org/stable/core/ops/trait.Deref.html#more-on-deref-coercion)
					// instead. The same applies to other lines following [`Pin::into_inner_unchecked`] in this file.
					.get(index)
					.map_unchecked(Pin::new_unchecked)
		}
	}

	/// Retrieves an exclusive reference to a pinned item,
	/// or [`None`] exactly whenever `index >= N`.
	#[must_use]
	pub fn get_mut(self: Pin<&mut Self>, index: usize) -> Option<Pin<&mut Item>> {
		unsafe {
			Pin::into_inner_unchecked(self)
				.get_mut(index)
				.map_unchecked(Pin::new_unchecked)
		}
	}

	/// Retrieves a shared reference to a pinned item without bounds-checking.
	#[must_use]
	pub fn get_unchecked(self: Pin<&Self>, index: usize) -> Pin<&Item> {
		unsafe {
			Pin::into_inner_unchecked(self)
						.get_unchecked(index)
						// Unsafe functions don't implement the closure traits even inside `unsafe` blocks.
						.pipe_unchecked(Pin::new_unchecked)
		}
	}

	/// Retrieves an exclusive reference to a pinned item without bounds-checking.
	#[must_use]
	pub fn get_unchecked_mut(self: Pin<&mut Self>, index: usize) -> Pin<&mut Item> {
		unsafe {
			Pin::into_inner_unchecked(self)
				.get_unchecked_mut(index)
				.pipe_unchecked(Pin::new_unchecked)
		}
	}
}

impl<'a, Item> IntoIterator for Pin<&'a PinnedPin<[Item]>> {
	type Item = Pin<&'a Item>;

	type IntoIter = Iter<'a, Item>;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { Pin::into_inner_unchecked(self) }.iter().pipe(Iter)
	}
}

impl<'a, Item> IntoIterator for Pin<&'a mut PinnedPin<[Item]>> {
	type Item = Pin<&'a mut Item>;

	type IntoIter = IterMut<'a, Item>;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { Pin::into_inner_unchecked(self) }
			.iter_mut()
			.pipe(IterMut)
	}
}

/// A sharing pinning slice iterator.
///
/// You can create one using `<Pin<&PinnedPin<[T]>> as IntoIterator>::into_iter`.
#[derive(Debug, Clone)]
pub struct Iter<'a, Item>(
	/// This field must not be public,
	/// as this (outer) pins the items through a borrow that doesn't directly guarantee pinning.
	core::slice::Iter<'a, Item>,
);

impl<'a, Item> Iterator for Iter<'a, Item> {
	type Item = Pin<&'a Item>;

	fn next(&mut self) -> Option<Self::Item> {
		unsafe { self.0.next().map_unchecked(Pin::new_unchecked) }
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

impl<'a, Item> DoubleEndedIterator for Iter<'a, Item> {
	fn next_back(&mut self) -> Option<Self::Item> {
		unsafe { self.0.next_back().map_unchecked(Pin::new_unchecked) }
	}
}

impl<'a, Item> ExactSizeIterator for Iter<'a, Item> {}
impl<'a, Item> FusedIterator for Iter<'a, Item> {}

/// An exclusive pinning slice iterator.
///
/// You can create one using `<Pin<&mut PinnedPin<[T]>> as IntoIterator>::into_iter`.
#[derive(Debug)]
pub struct IterMut<'a, Item>(
	/// This field must not be public,
	/// as this (outer) pins the items through a borrow that doesn't directly guarantee pinning.
	core::slice::IterMut<'a, Item>,
);

impl<'a, Item> Iterator for IterMut<'a, Item> {
	type Item = Pin<&'a mut Item>;

	fn next(&mut self) -> Option<Self::Item> {
		unsafe { self.0.next().map_unchecked(Pin::new_unchecked) }
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

impl<'a, Item> DoubleEndedIterator for IterMut<'a, Item> {
	fn next_back(&mut self) -> Option<Self::Item> {
		unsafe { self.0.next_back().map_unchecked(Pin::new_unchecked) }
	}
}

impl<'a, Item> ExactSizeIterator for IterMut<'a, Item> {}
impl<'a, Item> FusedIterator for IterMut<'a, Item> {}

/// We do need array versions of the `IntoIterator` implementations, for convenience at least.
/// Internally, these just use the slice iterators, like the standard library does.
impl<'a, Item, const N: usize> IntoIterator for Pin<&'a PinnedPin<[Item; N]>> {
	type Item = Pin<&'a Item>;

	type IntoIter = Iter<'a, Item>;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { Pin::into_inner_unchecked(self) }.iter().pipe(Iter)
	}
}

impl<'a, Item, const N: usize> IntoIterator for Pin<&'a mut PinnedPin<[Item; N]>> {
	type Item = Pin<&'a mut Item>;

	type IntoIter = IterMut<'a, Item>;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { Pin::into_inner_unchecked(self) }
			.iter_mut()
			.pipe(IterMut)
	}
}
