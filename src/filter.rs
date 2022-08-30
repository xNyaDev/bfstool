use std::fs::File;
use std::io::Read;

use globset::{GlobBuilder, GlobSetBuilder};

use crate::{Filter, string_lines_to_vec};

/// Load filters from file or filter name
pub fn load_filters(filter: Option<Filter>, file: Option<String>) -> Vec<String> {
    if let Some(file) = file {
        let mut file = File::open(file).expect("Failed to open filter file");
        let mut filters = String::new();
        file.read_to_string(&mut filters).unwrap();
        string_lines_to_vec(filters)
    } else {
        match filter.unwrap() {
            Filter::All => string_lines_to_vec(include_str!("filters/all.txt").to_string()),
            Filter::None => string_lines_to_vec(include_str!("filters/none.txt").to_string()),
            Filter::Fo1 => string_lines_to_vec(include_str!("filters/fo1.txt").to_string()),
            Filter::Fo2 => string_lines_to_vec(include_str!("filters/fo2.txt").to_string()),
            Filter::Fo2Demo => string_lines_to_vec(include_str!("filters/fo2-demo.txt").to_string()),
            Filter::Fo2Ps2Beta => string_lines_to_vec(include_str!("filters/fo2-ps2-beta.txt").to_string()),
            Filter::Fo2XboxBeta => string_lines_to_vec(include_str!("filters/fo2-xbox-beta.txt").to_string()),
            Filter::Fouc => string_lines_to_vec(include_str!("filters/fouc.txt").to_string()),
            Filter::FoucX360 => string_lines_to_vec(include_str!("filters/fouc-x360.txt").to_string()),
            Filter::Foho => string_lines_to_vec(include_str!("filters/foho.txt").to_string()),
            Filter::Srr => string_lines_to_vec(include_str!("filters/srr.txt").to_string()),
        }
    }
}

/// Apply filters to strings
pub fn apply_filters(strings: Vec<String>, filters: Vec<String>) -> Vec<String> {
    let mut filters_include = Vec::new();
    let mut glob_set_builder = GlobSetBuilder::new();
    // Exclude all comments
    let filters = filters.into_iter().filter(
        |filter| {
            !filter.starts_with("#")
        }
    ).collect::<Vec<String>>();
    // Build the filter glob set and keep which filters are include filters
    for filter_index in 0..filters.len() {
        if let Some(filter) = filters.get(filter_index) {
            if filter.starts_with("+ ") {
                filters_include.push(filter_index);
            }
            if !filter.starts_with("+ ") && !filter.starts_with("- ") {
                panic!("Invalid filter provided - Check README.md for filter details and examples");
            }
            let mut filter = filter.to_string();
            filter.remove(0);
            glob_set_builder.add(
                GlobBuilder::new(filter.trim()).literal_separator(true).build().expect(
                    &format!(
                        "Glob failed to parse: {}",
                        filter
                    )
                )
            );
        }
    }
    // Check last match for each string
    // If it's an include filter, the string should be included
    let glob_set = glob_set_builder.build().unwrap();
    let mut result = Vec::new();
    strings.into_iter().for_each(|string| {
        let mut matches_vec = glob_set.matches(&string);
        if let Some(match_index) = matches_vec.pop() {
            if filters_include.contains(&match_index) {
                result.push(string);
            }
        }
    });
    result
}

/// Apply single filter to strings
pub fn apply_single_filter(strings: Vec<String>, filter: String) -> Vec<String> {
    let glob = GlobBuilder::new(filter.trim()).literal_separator(true).build().expect(
        &format!(
            "Glob failed to parse: {}",
            filter
        )
    ).compile_matcher();

    strings.into_iter().filter(|string| {
        glob.is_match(string)
    }).collect()
}