//! Debug implementations for ranges of values
//!
//! This is a simple crate which helps debugging in certain scenarios. Many algorithms rely on
//! lists of items, such as integers, and often these lists contain runs of values that are
//! all "adjacent".
//!
//! For example, a filesystem implementation might store a list of block numbers that contain the
//! data for a particular file. If some blocks are allocated sequentially, then there may be
//! many runs of adjacent values. For example, `[42, 100, 101, 102, 103, 104, 20, 31, 32, 33, 34]`.
//! It can be helpful to display the runs as ranges, e.g. `[42, 100-104, 20, 31-34]`. This is more
//! compact and can help the developer spot patterns in data more quickly.
//!
//! This crate provides two types that display ranges more compactly, and functions which construct
//! those types.
//!
//! See [`debug_adjacent`] for an example.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::needless_lifetimes)]
#![cfg_attr(not(test), no_std)]

use core::fmt::{Debug, Formatter};

/// Returns a value that implements `Debug` by collapsing runs of "adjacent" items.
///
/// The `IsAdjacent` trait defines whether two values in `T` are adjacent. Implementations are
/// provided for Rust integer types.
///
/// # Example
/// ```
/// use dbg_ranges::debug_adjacent;
///
/// assert_eq!(
///     format!("{:?}", debug_adjacent(&[10u32, 12, 13, 14, 15, 20])),
///     "10, 12-15, 20"
/// );
/// ```
pub fn debug_adjacent<T: Debug + IsAdjacent>(items: &[T]) -> DebugAdjacent<T> {
    DebugAdjacent::new(items)
}

/// Returns a value that implements `Debug` by collapsing runs of "adjacent" items.
///
/// The `is_adjacent` parameter defines whether two values in `T` are adjacent.
pub fn debug_adjacent_by<T: Debug, F: Fn(&T, &T) -> bool>(
    items: &[T],
    is_adjacent: F,
) -> DebugAdjacentBy<T, F> {
    DebugAdjacentBy::new(items, is_adjacent)
}

macro_rules! int_successor {
    ($t:ty) => {
        impl IsAdjacent for $t {
            fn is_adjacent(&self, other: &Self) -> bool {
                other.checked_sub(*self) == Some(1)
            }
        }
    };
}
int_successor!(u8);
int_successor!(u16);
int_successor!(u32);
int_successor!(u64);
int_successor!(u128);
int_successor!(i8);
int_successor!(i16);
int_successor!(i32);
int_successor!(i64);
int_successor!(i128);

impl IsAdjacent for char {
    fn is_adjacent(&self, next: &Self) -> bool {
        if let Some(after_self) = (*self as u32).checked_add(1) {
            if let Some(after_self) = char::from_u32(after_self) {
                after_self == *next
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Checks whether an item is "adjacent" to another item.
///
/// ```
/// use dbg_ranges::IsAdjacent;
///
/// assert!(4.is_adjacent(&5));
/// ```
pub trait IsAdjacent {
    /// Returns `true` if `self` is adjacent to `other`.
    fn is_adjacent(&self, other: &Self) -> bool;
}

/// Displays a list of integers. If the list contains sequences of contiguous (increasing) values
/// then these will be displayed using `start-end` notation, rather than displaying each value.
///
/// The user of this type provides a function which indicates whether items are "adjacent" or not.
#[derive(Copy, Clone)]
pub struct DebugAdjacent<'a, T> {
    /// The items that will be displayed
    pub items: &'a [T],

    /// The separator between the first and last item in a range.
    pub sep: &'a str,
}

impl<'a, T> DebugAdjacent<'a, T> {
    /// Constructor
    pub fn new(items: &'a [T]) -> Self {
        Self { items, sep: "-" }
    }
}

impl<'a, T> Debug for DebugAdjacent<'a, T>
where
    T: Debug + IsAdjacent,
{
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let mut need_comma = false;

        let mut iter = self.items.iter().peekable();

        while let Some(first) = iter.next() {
            if need_comma {
                f.write_str(", ")?;
            }
            need_comma = true;

            let mut this: &T = first;
            let mut last: Option<&T> = None;

            while let Some(&next) = iter.peek() {
                if this.is_adjacent(next) {
                    this = next;
                    last = Some(next);
                    _ = iter.next();
                } else {
                    break;
                }
            }

            if let Some(last) = last {
                <T as Debug>::fmt(first, f)?;
                f.write_str(self.sep)?;
                <T as Debug>::fmt(last, f)?;
            } else {
                <T as Debug>::fmt(first, f)?;
            }
        }

        Ok(())
    }
}

/// Displays a list of integers. If the list contains sequences of contiguous (increasing) values
/// then these will be displayed using `start-end` notation, rather than displaying each value.
///
/// The user of this type provides a function which indicates whether items are "adjacent" or not.
#[derive(Copy, Clone)]
pub struct DebugAdjacentBy<'a, T, F> {
    /// The items that will be displayed
    pub items: &'a [T],
    /// The separator between the first and last item in a range.
    pub sep: &'a str,

    /// The function that tests for adjacency
    pub is_adjacent: F,
}

impl<'a, T, F> DebugAdjacentBy<'a, T, F> {
    /// Constructor
    pub fn new(items: &'a [T], is_adjacent: F) -> Self
    where
        F: Fn(&T, &T) -> bool,
    {
        Self {
            items,
            is_adjacent,
            sep: "-",
        }
    }
}

impl<'a, T, F> Debug for DebugAdjacentBy<'a, T, F>
where
    T: Debug,
    F: Fn(&T, &T) -> bool,
{
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let mut need_comma = false;

        let mut iter = self.items.iter().peekable();

        while let Some(first) = iter.next() {
            if need_comma {
                f.write_str(", ")?;
            }
            need_comma = true;

            let mut this: &T = first;
            let mut last: Option<&T> = None;

            while let Some(&next) = iter.peek() {
                if (self.is_adjacent)(this, next) {
                    this = next;
                    last = Some(next);
                    _ = iter.next();
                } else {
                    break;
                }
            }

            if let Some(last) = last {
                <T as Debug>::fmt(first, f)?;
                f.write_str(self.sep)?;
                <T as Debug>::fmt(last, f)?;
            } else {
                <T as Debug>::fmt(first, f)?;
            }
        }

        Ok(())
    }
}

#[test]
fn test_dump_ranges() {
    macro_rules! case {
        ($input:expr, $expected_output:expr) => {
            let input: &[_] = &$input;
            let dump = DebugAdjacent::new(input);
            let actual_output = format!("{:?}", dump);
            println!("dump_ranges: {:?} --> {:?}", input, actual_output);
            assert_eq!(
                actual_output.as_str(),
                $expected_output,
                "input: {:?}",
                input
            );
        };
    }

    case!([] as [u32; 0], "");
    case!([10u32], "10");
    case!([10u32, 20], "10, 20");
    case!([10u32, 11, 20], "10-11, 20");
    case!([10u32, 12, 13, 14, 15, 20], "10, 12-15, 20");
    case!([u32::MAX, 42], "4294967295, 42");
    case!([i32::MIN, i32::MIN + 1, 42], "-2147483648--2147483647, 42");
}

#[test]
fn test_dump_ranges_by() {
    macro_rules! case {
        ($input:expr, $expected_output:expr) => {
            let input: &[_] = &$input;
            let dump = DebugAdjacentBy::new(input, |&a, &b| a + 1 == b);
            let actual_output = format!("{:?}", dump);
            println!("dump_ranges: {:?} --> {:?}", input, actual_output);
            assert_eq!(
                actual_output.as_str(),
                $expected_output,
                "input: {:?}",
                input
            );
        };
    }

    case!([] as [u32; 0], "");
    case!([10u32], "10");
    case!([10u32, 20], "10, 20");
    case!([10u32, 11, 20], "10-11, 20");
    case!([10u32, 12, 13, 14, 15, 20], "10, 12-15, 20");
}

#[test]
fn test_dump_ranges_by_swapped() {
    macro_rules! case {
        ($input:expr, $expected_output:expr) => {
            let input: &[_] = &$input;
            let dump = DebugAdjacentBy::new(input, |&a, &b| a + 1 == b);
            let actual_output = format!("{:?}", dump);
            println!("dump_ranges: {:?} --> {:?}", input, actual_output);
            assert_eq!(
                actual_output.as_str(),
                $expected_output,
                "input: {:?}",
                input
            );
        };
    }

    case!([] as [u32; 0], "");
    case!([10u32], "10");
    case!([10u32, 20], "10, 20");
    case!([10u32, 11, 20], "10-11, 20");
    case!([10u32, 12, 13, 14, 15, 20], "10, 12-15, 20");
}
