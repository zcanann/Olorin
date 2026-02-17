use std::ops::Range;

/// Builds a bounded viewport range for list rows using selection-relative windowing.
pub fn build_selection_relative_viewport_range(
    total_entry_count: usize,
    selected_entry_index: Option<usize>,
    viewport_capacity: usize,
) -> Range<usize> {
    if total_entry_count == 0 || viewport_capacity == 0 {
        return 0..0;
    }

    let visible_entry_count = total_entry_count.min(viewport_capacity);
    if visible_entry_count == total_entry_count {
        return 0..total_entry_count;
    }

    let selected_entry_index = selected_entry_index
        .filter(|selected_entry_index| *selected_entry_index < total_entry_count)
        .unwrap_or(0);
    let leading_context_count = visible_entry_count / 2;
    let mut viewport_start_index = selected_entry_index.saturating_sub(leading_context_count);
    let maximum_viewport_start_index = total_entry_count.saturating_sub(visible_entry_count);
    if viewport_start_index > maximum_viewport_start_index {
        viewport_start_index = maximum_viewport_start_index;
    }

    let viewport_end_index = viewport_start_index + visible_entry_count;
    viewport_start_index..viewport_end_index
}

#[cfg(test)]
mod tests {
    use crate::views::entry_row_viewport::build_selection_relative_viewport_range;

    #[test]
    fn viewport_defaults_to_top_when_selection_missing() {
        assert_eq!(build_selection_relative_viewport_range(20, None, 5), 0..5);
    }

    #[test]
    fn viewport_centers_around_middle_selection() {
        assert_eq!(build_selection_relative_viewport_range(20, Some(10), 5), 8..13);
    }

    #[test]
    fn viewport_clamps_to_bottom_for_tail_selection() {
        assert_eq!(build_selection_relative_viewport_range(20, Some(19), 5), 15..20);
    }

    #[test]
    fn viewport_clamps_invalid_selection_to_top() {
        assert_eq!(build_selection_relative_viewport_range(20, Some(99), 5), 0..5);
    }
}
