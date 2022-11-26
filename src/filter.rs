use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use globset::{GlobBuilder, GlobSetBuilder};
use regex::Regex;

use crate::{CopyFilter, Filter, string_lines_to_vec};

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
            Filter::Fo2FxPatch => string_lines_to_vec(include_str!("filters/fo2-fx-patch.txt").to_string()),
            Filter::Fo2Demo => string_lines_to_vec(include_str!("filters/fo2-demo.txt").to_string()),
            Filter::Fo2Ps2Beta => string_lines_to_vec(include_str!("filters/fo2-ps2-beta.txt").to_string()),
            Filter::Fo2XboxBeta => string_lines_to_vec(include_str!("filters/fo2-xbox-beta.txt").to_string()),
            Filter::Fouc => string_lines_to_vec(include_str!("filters/fouc.txt").to_string()),
            Filter::FoucX360 => string_lines_to_vec(include_str!("filters/fouc-x360.txt").to_string()),
            Filter::Foho => string_lines_to_vec(include_str!("filters/foho.txt").to_string()),
            Filter::Srr => string_lines_to_vec(include_str!("filters/srr.txt").to_string()),
            Filter::Rru => string_lines_to_vec(include_str!("filters/rru.txt").to_string()),
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

/// Apply copy filters to strings
pub fn apply_copy_filters(strings: Vec<String>, filters: Vec<String>) -> HashMap<String, (u8, u16)> {
    let mut filters_values = HashMap::new();
    let mut glob_set_builder = GlobSetBuilder::new();
    let regex_check = Regex::new(r"^\d+\+\d+ .+$").unwrap();
    // Exclude all comments
    let filters = filters.into_iter().filter(
        |filter| {
            !filter.starts_with("#")
        }
    ).collect::<Vec<String>>();
    // Build the filter glob set and filter map
    for filter_index in 0..filters.len() {
        if let Some(filter) = filters.get(filter_index) {
            if !regex_check.is_match(filter) {
                panic!("Invalid copy filter provided - Check README.md for copy filter details and examples");
            }
            let split_position = filter.find(' ').unwrap();
            let copy_info = filter[..split_position].chars().collect::<String>();
            let copy_info = copy_info.split('+').collect::<Vec<&str>>();
            let copy_info = (
                u8::from_str_radix(copy_info[0], 10).unwrap(),
                u16::from_str_radix(copy_info[1], 10).unwrap()
            );
            filters_values.insert(filter_index, copy_info);
            let filter = filter[split_position..].chars().collect::<String>();
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
    // If there's none, treat it like 0+0
    let glob_set = glob_set_builder.build().unwrap();
    let mut result = HashMap::new();
    strings.into_iter().for_each(|string| {
        let mut matches_vec = glob_set.matches(&string);
        if let Some(match_index) = matches_vec.pop() {
            let copy_info = filters_values.get(&match_index).unwrap().clone();
            result.insert(string, copy_info);
        } else {
            result.insert(string, (0, 0));
        }
    });
    result
}

/// Load copy filters from file or filter name
pub fn load_copy_filters(filter: Option<CopyFilter>, file: Option<String>) -> Vec<String> {
    if let Some(file) = file {
        let mut file = File::open(file).expect("Failed to open copy filter file");
        let mut filters = String::new();
        file.read_to_string(&mut filters).unwrap();
        string_lines_to_vec(filters)
    } else {
        match filter.unwrap() {
            CopyFilter::None => string_lines_to_vec(include_str!("copy_filters/none.txt").to_string()),
            CopyFilter::Fo1Pc => string_lines_to_vec(include_str!("copy_filters/fo1-pc.txt").to_string()),
            CopyFilter::Fo1Ps2 => string_lines_to_vec(include_str!("copy_filters/fo1-ps2.txt").to_string()),
            CopyFilter::Fo1Ps2Jp => string_lines_to_vec(include_str!("copy_filters/fo1-ps2-jp.txt").to_string()),
            CopyFilter::Fo1Ps2Usa => string_lines_to_vec(include_str!("copy_filters/fo1-ps2-usa.txt").to_string()),
            CopyFilter::Fo1Xbox => string_lines_to_vec(include_str!("copy_filters/fo1-xbox.txt").to_string()),
            CopyFilter::Fo2Pc => string_lines_to_vec(include_str!("copy_filters/fo2-pc.txt").to_string()),
            CopyFilter::Fo2Ps2 => string_lines_to_vec(include_str!("copy_filters/fo2-ps2.txt").to_string()),
            CopyFilter::Fo2Ps2Beta => string_lines_to_vec(include_str!("copy_filters/fo2-ps2-beta.txt").to_string()),
            CopyFilter::Fo2Ps2GermanPack => string_lines_to_vec(include_str!("copy_filters/fo2-ps2-german-pack.txt").to_string()),
            CopyFilter::Fo2Ps2Usa => string_lines_to_vec(include_str!("copy_filters/fo2-ps2-usa.txt").to_string()),
            CopyFilter::Fo2Xbox => string_lines_to_vec(include_str!("copy_filters/fo2-xbox.txt").to_string()),
            CopyFilter::Fo2XboxBeta => string_lines_to_vec(include_str!("copy_filters/fo2-xbox-beta.txt").to_string()),
            CopyFilter::FoucPc => string_lines_to_vec(include_str!("copy_filters/fouc-pc.txt").to_string()),
            CopyFilter::FoucPcLangpack => string_lines_to_vec(include_str!("copy_filters/fouc-pc-langpack.txt").to_string()),
            CopyFilter::FoucX360 => string_lines_to_vec(include_str!("copy_filters/fouc-x360.txt").to_string()),
            CopyFilter::FoucX360De => string_lines_to_vec(include_str!("copy_filters/fouc-x360-de.txt").to_string()),
            CopyFilter::FoucX360Jp => string_lines_to_vec(include_str!("copy_filters/fouc-x360-jp.txt").to_string()),
            CopyFilter::Foho => string_lines_to_vec(include_str!("copy_filters/foho.txt").to_string()),
            CopyFilter::Srr => string_lines_to_vec(include_str!("copy_filters/srr.txt").to_string()),
            CopyFilter::Rru => string_lines_to_vec(include_str!("copy_filters/rru.txt").to_string()),
            CopyFilter::RruPcUpdate => string_lines_to_vec(include_str!("copy_filters/rru-pc-update.txt").to_string()),
        }
    }
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