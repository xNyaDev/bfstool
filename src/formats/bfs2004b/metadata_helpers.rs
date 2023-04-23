use crate::formats::bfs2004b::{HashTable, MetadataHeader};

pub fn calculate_metadata_count(
    wanted_start: u32,
    metadata_header: &MetadataHeader,
    header_end: u32,
    metadata_start: u32,
) -> usize {
    let corrected_header = MetadataHeader {
        file_headers_offset: metadata_header.file_headers_offset + metadata_start,
        file_name_offset_table_offset: metadata_header.file_name_offset_table_offset
            + metadata_start,
        file_name_length_table_offset: metadata_header.file_name_length_table_offset
            + metadata_start,
        huffman_dictionary_offset: metadata_header.huffman_dictionary_offset + metadata_start,
        huffman_data_offset: metadata_header.huffman_data_offset + metadata_start,
    };

    let corrected_wanted_start = wanted_start + metadata_start;

    let mut offsets = vec![
        header_end,
        corrected_header.file_headers_offset,
        corrected_header.file_name_offset_table_offset,
        corrected_header.file_name_length_table_offset,
        corrected_header.huffman_dictionary_offset,
        corrected_header.huffman_data_offset,
    ];

    offsets.sort();

    let mut wanted_end = 0;

    offsets
        .iter()
        .zip(offsets.iter().skip(1))
        .for_each(|(offset, next_offset)| {
            if offset == &corrected_wanted_start {
                wanted_end = *next_offset;
            }
        });

    if corrected_wanted_start == corrected_header.file_name_offset_table_offset {
        ((wanted_end - corrected_wanted_start) / 4) as usize
    } else if corrected_wanted_start == corrected_header.file_name_length_table_offset
        || corrected_wanted_start == corrected_header.huffman_dictionary_offset
    {
        ((wanted_end - corrected_wanted_start) / 2) as usize
    } else if corrected_wanted_start == corrected_header.huffman_data_offset {
        (wanted_end - corrected_wanted_start) as usize
    } else {
        0
    }
}

pub fn calculate_metadata_start(hash_table: &HashTable) -> u32 {
    hash_table.entries.len() as u32 * 8 + 20
}
