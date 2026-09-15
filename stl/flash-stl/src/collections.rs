//! STL collections — List, Map, Set operations for `.ui` handlers.

/// Collection algorithms available in `.ui` via Tier-1 or built-in where static.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollectionFn {
    Len,
    IsEmpty,
    First,
    Last,
    Contains,
    IndexOf,
    Filter,
    Map,
    Reduce,
    SortBy,
    Reverse,
    Take,
    Skip,
    Chunk,
    FlatMap,
    Distinct,
    Zip,
}

impl CollectionFn {
    pub fn name(&self) -> &'static str {
        match self {
            CollectionFn::Len => "len",
            CollectionFn::IsEmpty => "is_empty",
            CollectionFn::First => "first",
            CollectionFn::Last => "last",
            CollectionFn::Contains => "contains",
            CollectionFn::IndexOf => "index_of",
            CollectionFn::Filter => "filter",
            CollectionFn::Map => "map",
            CollectionFn::Reduce => "reduce",
            CollectionFn::SortBy => "sort_by",
            CollectionFn::Reverse => "reverse",
            CollectionFn::Take => "take",
            CollectionFn::Skip => "skip",
            CollectionFn::Chunk => "chunk",
            CollectionFn::FlatMap => "flat_map",
            CollectionFn::Distinct => "distinct",
            CollectionFn::Zip => "zip",
        }
    }

    pub fn doc(&self) -> &'static str {
        match self {
            CollectionFn::Len => "Returns the number of elements.",
            CollectionFn::IsEmpty => "Returns true if the collection has no elements.",
            CollectionFn::First => "Returns the first element, or none.",
            CollectionFn::Last => "Returns the last element, or none.",
            CollectionFn::Contains => "Returns true if the item exists in the collection.",
            CollectionFn::IndexOf => "Returns the index of the item, or -1.",
            CollectionFn::Filter => "Returns elements matching the predicate.",
            CollectionFn::Map => "Transforms each element.",
            CollectionFn::Reduce => "Reduces the collection to a single value.",
            CollectionFn::SortBy => "Sorts by a key function.",
            CollectionFn::Reverse => "Reverses element order.",
            CollectionFn::Take => "Returns the first n elements.",
            CollectionFn::Skip => "Skips the first n elements.",
            CollectionFn::Chunk => "Splits into fixed-size chunks.",
            CollectionFn::FlatMap => "Maps and flattens one level.",
            CollectionFn::Distinct => "Removes duplicate elements.",
            CollectionFn::Zip => "Pairs two collections element-wise.",
        }
    }

    pub fn all() -> &'static [CollectionFn] {
        &[
            CollectionFn::Len, CollectionFn::IsEmpty, CollectionFn::First, CollectionFn::Last,
            CollectionFn::Contains, CollectionFn::IndexOf, CollectionFn::Filter, CollectionFn::Map,
            CollectionFn::Reduce, CollectionFn::SortBy, CollectionFn::Reverse, CollectionFn::Take,
            CollectionFn::Skip, CollectionFn::Chunk, CollectionFn::FlatMap, CollectionFn::Distinct,
            CollectionFn::Zip,
        ]
    }
}
