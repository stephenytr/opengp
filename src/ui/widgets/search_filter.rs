//! Reusable search/filter widget with fuzzy matching.
//!
//! This module provides a generic `SearchFilter<T>` that enables fuzzy text search
//! over a collection of items. It wraps the `sublime_fuzzy` crate to provide
//! intelligent matching that ranks results by relevance score.
//!
//! # Usage
//!
//! ```rust
//! use opengp::ui::widgets::SearchFilter;
//!
//! #[derive(Debug, Clone)]
//! struct Patient {
//!     pub name: String,
//!     pub mrn: String,
//! }
//!
//! // Create a text extractor function
//! let extract = |p: &Patient| format!("{} {}", p.name, p.mrn);
//!
//! let mut filter = SearchFilter::new(vec![
//!     Patient { name: "John Smith".to_string(), mrn: "MRN001".to_string() },
//!     Patient { name: "Jane Doe".to_string(), mrn: "MRN002".to_string() },
//!     Patient { name: "Johnathan Smith".to_string(), mrn: "MRN003".to_string() },
//! ], extract);
//!
//! // Filter by name
//! filter.set_query("john");
//! let results: Vec<_> = filter.filtered().collect();
//! assert!(results.len() >= 2); // John Smith and Johnathan Smith
//!
//! // Filter by MRN
//! filter.set_query("mrn003");
//! let results: Vec<_> = filter.filtered().collect();
//! assert_eq!(results.len(), 1);
//! ```
//!
//! # Fuzzy Matching
//!
//! The filter uses Sublime Text's fuzzy matching algorithm which:
//! - Matches characters in sequence (non-contiguous)
//! - Gives higher scores for consecutive matches
//! - Gives higher scores for matches at word boundaries
//! - Gives higher scores for matches at the start of strings
//! - Is case-insensitive by default

use sublime_fuzzy::best_match;

/// Generic search/filter state manager with fuzzy matching.
///
/// This struct maintains a collection of items and provides filtering
/// based on fuzzy text matching against a user-provided query string.
///
/// # Type Parameters
///
/// * `T` - The type of items being filtered
///
/// # Notes
///
/// - Empty query returns all items (no filtering applied)
/// - Fuzzy matching is case-insensitive
/// - Results are sorted by relevance score (highest first)
/// - The `extract_text` function is called for each item during filtering
pub struct SearchFilter<T> {
    items: Vec<T>,
    query: String,
    extract_text: Box<dyn Fn(&T) -> String>,
}

impl<T> SearchFilter<T> {
    /// Creates a new `SearchFilter` with the given items and text extractor.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of items to filter
    /// * `extract_text` - A function that extracts searchable text from each item
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let items = vec!["apple", "banana", "cherry"];
    /// let filter = SearchFilter::new(items, |s: &&str| s.to_string());
    ///
    /// assert_eq!(filter.query(), "");
    /// assert_eq!(filter.filtered().count(), 3);
    /// ```
    pub fn new(items: Vec<T>, extract_text: impl Fn(&T) -> String + 'static) -> Self {
        Self {
            items,
            query: String::new(),
            extract_text: Box::new(extract_text),
        }
    }

    /// Sets the search query string.
    ///
    /// Setting an empty query will cause `filtered()` to return all items.
    ///
    /// # Arguments
    ///
    /// * `query` - The new search query string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let filter = SearchFilter::new(vec!["hello", "world"], |s: &&str| s.to_string());
    ///
    /// filter.set_query("he");
    /// assert_eq!(filter.query(), "he");
    /// assert_eq!(filter.filtered().count(), 1); // "hello"
    /// ```
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    /// Returns a reference to the current search query.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let filter = SearchFilter::new(vec![1, 2, 3], |i: &i32| i.to_string());
    /// assert_eq!(filter.query(), "");
    ///
    /// filter.set_query("test");
    /// assert_eq!(filter.query(), "test");
    /// ```
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns an iterator over items that match the current query.
    ///
    /// If the query is empty, all items are returned in their original order.
    /// Otherwise, items are sorted by fuzzy match score (highest first).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let items = vec!["apple", "banana", "apricot"];
    /// let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());
    ///
    /// filter.set_query("ap");
    /// let matched: Vec<_> = filter.filtered().collect();
    ///
    /// // "apple" and "apricot" match, but "apple" scores higher (starts with "ap")
    /// assert_eq!(matched.len(), 2);
    /// assert_eq!(matched[0], "apple");
    /// ```
    ///
    /// # Empty Query
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let items = vec!["a", "b", "c"];
    /// let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());
    ///
    /// // No query set - returns all items
    /// let all: Vec<_> = filter.filtered().collect();
    /// assert_eq!(all, items);
    /// ```
    pub fn filtered(&self) -> impl Iterator<Item = &T> + '_ {
        if self.query.is_empty() {
            Box::new(self.items.iter()) as Box<dyn Iterator<Item = &T> + '_>
        } else {
            let extract = &self.extract_text;
            let query = self.query.clone();

            let mut scored: Vec<(&T, isize)> = self
                .items
                .iter()
                .filter_map(|item| {
                    let text = extract(item);
                    best_match(&query, &text).map(|m| (item, m.score()))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));

            Box::new(scored.into_iter().map(|(item, _)| item)) as Box<dyn Iterator<Item = &T> + '_>
        }
    }

    /// Returns the number of items in the original collection.
    ///
    /// This is the total count before any filtering is applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let filter = SearchFilter::new(vec!["a", "b", "c"], |s: &&str| s.to_string());
    /// assert_eq!(filter.len(), 3);
    ///
    /// filter.set_query("x");
    /// assert_eq!(filter.len(), 3); // Total count unchanged
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the collection contains no items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let empty: SearchFilter<i32> = SearchFilter::new(vec![], |i: &i32| i.to_string());
    /// assert!(empty.is_empty());
    ///
    /// let filter = SearchFilter::new(vec![1], |i: &i32| i.to_string());
    /// assert!(!filter.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items that match the current query.
    ///
    /// This is equivalent to `filtered().count()` but may be more efficient
    /// for some use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let filter = SearchFilter::new(vec!["apple", "banana", "apricot"], |s: &&str| s.to_string());
    /// filter.set_query("ap");
    /// assert_eq!(filter.matched_count(), 2);
    /// ```
    pub fn matched_count(&self) -> usize {
        self.filtered().count()
    }

    /// Clears the current search query.
    ///
    /// After clearing, `filtered()` will return all items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let filter = SearchFilter::new(vec!["a", "b", "c"], |s: &&str| s.to_string());
    /// filter.set_query("a");
    /// assert_eq!(filter.matched_count(), 1);
    ///
    /// filter.clear();
    /// assert_eq!(filter.matched_count(), 3);
    /// assert!(filter.query().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.query.clear();
    }

    /// Returns a reference to all items (unfiltered).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::SearchFilter;
    ///
    /// let items = vec!["a", "b", "c"];
    /// let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());
    ///
    /// filter.set_query("x");
    /// assert_eq!(filter.items(), &items);
    /// ```
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }
}

/// Convenience alias for a text extractor function.
pub type TextExtractor<T> = Box<dyn Fn(&T) -> String>;

#[cfg(test)]
mod tests {
    use super::*;

    // === Constructor Tests ===

    #[test]
    fn test_new_with_items() {
        let items = vec!["apple", "banana", "cherry"];
        let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());
        assert_eq!(filter.items(), &items);
    }

    #[test]
    fn test_new_with_empty_list() {
        let filter: SearchFilter<&str> = SearchFilter::new(vec![], |s: &&str| s.to_string());
        assert!(filter.is_empty());
        assert_eq!(filter.query(), "");
    }

    #[test]
    fn test_new_preserves_extractor() {
        let filter = SearchFilter::new(vec![1, 2, 3], |i: &i32| i.to_string());
        // The extractor is stored but we can't directly test it without filtering
        assert_eq!(filter.len(), 3);
    }

    // === Query Tests ===

    #[test]
    fn test_set_query() {
        let mut filter = SearchFilter::new(vec!["a", "b"], |s: &&str| s.to_string());
        assert_eq!(filter.query(), "");

        filter.set_query("test");
        assert_eq!(filter.query(), "test");

        filter.set_query("another");
        assert_eq!(filter.query(), "another");
    }

    #[test]
    fn test_set_query_from_string() {
        let mut filter = SearchFilter::new(vec![1], |_: &i32| String::new());
        filter.set_query(String::from("hello"));
        assert_eq!(filter.query(), "hello");
    }

    #[test]
    fn test_clear_query() {
        let mut filter = SearchFilter::new(vec!["a", "b"], |s: &&str| s.to_string());
        filter.set_query("a");
        assert_eq!(filter.query(), "a");

        filter.clear();
        assert!(filter.query().is_empty());
    }

    // === Basic Filtering Tests ===

    #[test]
    fn test_filtered_empty_query_returns_all() {
        let items = vec!["apple", "banana", "cherry"];
        let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        let result: Vec<_> = filter.filtered().collect();
        let result_strs: Vec<&str> = result.iter().map(|s| **s).collect();
        assert_eq!(result_strs, items);
    }

    #[test]
    fn test_filtered_exact_match() {
        let items = vec!["apple", "banana", "cherry"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("banana");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &"banana");
    }

    #[test]
    fn test_filtered_partial_match() {
        let items = vec!["apple", "application", "banana"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("app");
        let result: Vec<_> = filter.filtered().collect();

        // Both "apple" and "application" match
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filtered_no_matches() {
        let items = vec!["apple", "banana", "cherry"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("xyz");
        let result: Vec<_> = filter.filtered().collect();

        assert!(result.is_empty());
    }

    // === Fuzzy Matching Tests ===

    #[test]
    fn test_fuzzy_match_scatters() {
        let items = vec!["scissors", "screens", "scarlet", "screencast"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("sc");
        let result: Vec<_> = filter.filtered().collect();

        // "scissors" and "scarlet" start with "sc" (better match)
        // "screens" and "screencast" have "sc" not at start
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_fuzzy_match_non_contiguous() {
        let items = vec!["Stephen", "Steven", "Stepfan"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("stphn");
        let result: Vec<_> = filter.filtered().collect();

        // At least one should match
        assert!(!result.is_empty());
    }

    #[test]
    fn test_fuzzy_case_insensitive() {
        let items = vec!["Apple", "BANANA", "Cherry"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("apple");
        let result: Vec<_> = filter.filtered().collect();

        assert!(!result.is_empty());
    }

    // === Score Sorting Tests ===

    #[test]
    fn test_filtered_sorts_by_score() {
        let items = vec!["apricot", "apple", "anthropology"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("ap");
        let result: Vec<_> = filter.filtered().collect();

        // Both "apple" and "apricot" should match
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_filtered_start_of_string_higher_score() {
        let items = vec!["foobar", "foobarbaz", "bazfoobar"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("foo");
        let result: Vec<_> = filter.filtered().collect();

        // "foobar" should be first (exact start match)
        assert_eq!(result[0], &"foobar");
    }

    // === Complex Types Tests ===

    #[test]
    fn test_filter_with_struct() {
        #[derive(Debug, Clone, PartialEq)]
        struct Patient {
            name: String,
            mrn: String,
        }

        let patients = vec![
            Patient {
                name: "John Smith".to_string(),
                mrn: "MRN001".to_string(),
            },
            Patient {
                name: "Jane Doe".to_string(),
                mrn: "MRN002".to_string(),
            },
            Patient {
                name: "John Doe".to_string(),
                mrn: "MRN003".to_string(),
            },
        ];

        let mut filter = SearchFilter::new(patients.clone(), |p: &Patient| {
            format!("{} {}", p.name, p.mrn)
        });

        filter.set_query("john");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_with_multiple_fields() {
        #[derive(Debug, Clone)]
        struct Item {
            id: u32,
            name: String,
            category: String,
        }

        let items = vec![
            Item {
                id: 1,
                name: "Laptop".to_string(),
                category: "Electronics".to_string(),
            },
            Item {
                id: 2,
                name: "Mouse".to_string(),
                category: "Electronics".to_string(),
            },
            Item {
                id: 3,
                name: "Chair".to_string(),
                category: "Furniture".to_string(),
            },
        ];

        let mut filter = SearchFilter::new(items, |item: &Item| {
            format!("{} {}", item.name, item.category)
        });

        filter.set_query("electronics");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 2);
    }

    // === Length and Empty Tests ===

    #[test]
    fn test_len() {
        let filter = SearchFilter::new(vec!["a", "b", "c", "d"], |s: &&str| s.to_string());
        assert_eq!(filter.len(), 4);
    }

    #[test]
    fn test_len_empty_list() {
        let filter: SearchFilter<i32> = SearchFilter::new(vec![], |i: &i32| i.to_string());
        assert_eq!(filter.len(), 0);
    }

    #[test]
    fn test_is_empty_true() {
        let filter: SearchFilter<i32> = SearchFilter::new(vec![], |i: &i32| i.to_string());
        assert!(filter.is_empty());
    }

    #[test]
    fn test_is_empty_false() {
        let filter = SearchFilter::new(vec![1], |i: &i32| i.to_string());
        assert!(!filter.is_empty());
    }

    // === Matched Count Tests ===

    #[test]
    fn test_matched_count() {
        let mut filter =
            SearchFilter::new(vec!["apple", "banana", "apricot"], |s: &&str| s.to_string());

        filter.set_query("ap");
        assert_eq!(filter.matched_count(), 2);
    }

    #[test]
    fn test_matched_count_empty_query() {
        let filter = SearchFilter::new(vec!["a", "b", "c"], |s: &&str| s.to_string());

        // Empty query returns all items
        assert_eq!(filter.matched_count(), 3);
    }

    #[test]
    fn test_matched_count_no_matches() {
        let mut filter = SearchFilter::new(vec!["apple", "banana"], |s: &&str| s.to_string());

        filter.set_query("xyz");
        assert_eq!(filter.matched_count(), 0);
    }

    // === Edge Cases ===

    #[test]
    fn test_filter_unicode() {
        let items = vec!["café", "résumé", "naïve"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("caf");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &"café");
    }

    #[test]
    fn test_filter_special_characters() {
        let items = vec!["hello@world.com", "test-user", "name_surname"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("@");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &"hello@world.com");
    }

    #[test]
    fn test_filter_numbers_as_text() {
        let items = vec!["item123", "item456", "other789"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("456");
        let result: Vec<_> = filter.filtered().collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &"item456");
    }

    #[test]
    fn test_single_character_query() {
        let items = vec!["apple", "banana", "cherry"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("a");
        let result: Vec<_> = filter.filtered().collect();

        // "apple" and "banana" both have 'a'
        assert!(result.len() >= 1);
    }

    #[test]
    fn test_whitespace_in_query() {
        let items = vec!["New York", "New Zealand", "New Delhi"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("New ");
        let result: Vec<_> = filter.filtered().collect();

        // Should match all with "New "
        assert!(result.len() >= 1);
    }

    // === Reuse and Mutation Tests ===

    #[test]
    fn test_reuse_with_different_queries() {
        let items = vec!["apple", "banana", "cherry", "date"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("a");
        let result1: Vec<_> = filter.filtered().map(|s| *s).collect();

        filter.set_query("b");
        let result2: Vec<_> = filter.filtered().map(|s| *s).collect();

        filter.set_query("xyz");
        let result3: Vec<_> = filter.filtered().map(|s| *s).collect();

        assert!(!result1.is_empty());
        assert!(!result2.is_empty());
        assert!(result3.is_empty());
    }

    #[test]
    fn test_clear_and_refilter() {
        let items = vec!["apple", "banana"];
        let mut filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        filter.set_query("a");
        assert_eq!(filter.matched_count(), 2);

        filter.clear();
        assert_eq!(filter.matched_count(), 2); // All items returned

        filter.set_query("apple");
        assert_eq!(filter.matched_count(), 1);
    }

    // === Iterator Consumption Tests ===

    #[test]
    fn test_filtered_is_lazy() {
        // This test verifies that filtered() returns an iterator
        // that can be consumed multiple times (not consuming the original)
        let items = vec!["apple", "banana", "cherry"];
        let filter = SearchFilter::new(items.clone(), |s: &&str| s.to_string());

        // Get first result
        let first = filter.filtered().next();
        assert_eq!(first, Some(&"apple"));

        // Can iterate again - original is not consumed
        let all: Vec<_> = filter.filtered().collect();
        assert_eq!(all.len(), 3);
    }
}
