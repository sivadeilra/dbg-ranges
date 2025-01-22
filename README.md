# Debugging ranges of sequential values

This is a simple crate which helps debugging in certain scenarios. Many algorithms rely on
lists of items, such as integers, and often these lists contain runs of values that are
all "adjacent".

For example, a filesystem implementation might store a list of block numbers that contain the
data for a particular file. If some blocks are allocated sequentially, then there may be
many runs of adjacent values. For example, `[42, 100, 101, 102, 103, 104, 20, 31, 32, 33, 34]`.
It can be helpful to display the runs as ranges, e.g. `[42, 100-104, 20, 31-34]`. This is more
compact and can help the developer spot patterns in data more quickly.

This crate provides two types that display ranges more compactly, and functions which construct
those types.
