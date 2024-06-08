use std::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeTo};
use std::slice;

#[repr(C)]
pub struct TArray<T> {
	pub Data: *mut T,
	ArrayNum: i32,
	ArrayMax: i32,
}

impl<T> TArray<T> {
	#[must_use]
	pub fn as_slice(&self) -> &[T] {
		unsafe { slice::from_raw_parts(self.Data, self.ArrayNum as usize) }
	}

	#[must_use]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe { slice::from_raw_parts_mut(self.Data, self.ArrayMax as usize) }
	}

	#[must_use]
	pub fn get(&self, index: i32) -> Option<&T> {
		self.deref().get(index as usize)
	}

	#[must_use]
	pub fn get_mut(&mut self, index: i32) -> Option<&mut T> {
		self.deref_mut().get_mut(index as usize)
	}
}

impl<T> Deref for TArray<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<T> DerefMut for TArray<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

impl<T> Index<i32> for TArray<T> {
	type Output = T;

	fn index(&self, index: i32) -> &Self::Output {
		&self.as_slice()[index as usize]
	}
}

impl<T> IndexMut<i32> for TArray<T> {
	fn index_mut(&mut self, index: i32) -> &mut Self::Output {
		&mut self.as_mut_slice()[index as usize]
	}
}

impl<T> Index<Range<i32>> for TArray<T> {
	type Output = [T];

	fn index(&self, Range { start, end }: Range<i32>) -> &Self::Output {
		&self.as_slice()[start as usize..end as usize]
	}
}

impl<T> IndexMut<Range<i32>> for TArray<T> {
	fn index_mut(&mut self, Range { start, end }: Range<i32>) -> &mut Self::Output {
		&mut self.as_mut_slice()[start as usize..end as usize]
	}
}

impl<T> Index<RangeFrom<i32>> for TArray<T> {
	type Output = [T];

	fn index(&self, RangeFrom { start }: RangeFrom<i32>) -> &Self::Output {
		&self.as_slice()[start as usize..]
	}
}

impl<T> IndexMut<RangeFrom<i32>> for TArray<T> {
	fn index_mut(&mut self, RangeFrom { start }: RangeFrom<i32>) -> &mut Self::Output {
		&mut self.as_mut_slice()[start as usize..]
	}
}

impl<T> Index<RangeTo<i32>> for TArray<T> {
	type Output = [T];

	fn index(&self, RangeTo { end }: RangeTo<i32>) -> &Self::Output {
		&self.as_slice()[..end as usize]
	}
}

impl<T> IndexMut<RangeTo<i32>> for TArray<T> {
	fn index_mut(&mut self, RangeTo { end }: RangeTo<i32>) -> &mut Self::Output {
		&mut self.as_mut_slice()[..end as usize]
	}
}

impl<'a, T> IntoIterator for &'a TArray<T> {
	type IntoIter = slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
