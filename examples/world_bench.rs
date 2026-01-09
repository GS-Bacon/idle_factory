//! Benchmark for world generation and memory usage

use bevy::prelude::IVec2;
use idle_factory::constants::{CHUNK_HEIGHT, GROUND_LEVEL, SECTIONS_PER_CHUNK, SECTION_HEIGHT};
use idle_factory::world::{ChunkData, ChunkSection};
use std::time::Instant;

fn main() {
    println!("=== World Optimization Benchmark ===\n");

    // Test parameters
    let view_distances = [3, 5, 7, 10];

    for &view_dist in &view_distances {
        let chunk_count = (view_dist * 2 + 1) * (view_dist * 2 + 1);
        println!(
            "--- VIEW_DISTANCE={} ({} chunks) ---",
            view_dist, chunk_count
        );

        // Generate chunks
        let start = Instant::now();
        let mut chunks: Vec<ChunkData> = Vec::with_capacity(chunk_count as usize);

        for x in -view_dist..=view_dist {
            for z in -view_dist..=view_dist {
                chunks.push(ChunkData::generate(IVec2::new(x, z)));
            }
        }
        let gen_time = start.elapsed();

        // Calculate memory usage
        let mut total_memory = 0usize;
        let mut empty_sections = 0usize;
        let mut uniform_sections = 0usize;
        let mut paletted_sections = 0usize;

        for chunk in &chunks {
            total_memory += chunk.memory_usage();
            for section in chunk.sections() {
                match section {
                    ChunkSection::Empty => empty_sections += 1,
                    ChunkSection::Uniform(_) => uniform_sections += 1,
                    ChunkSection::Paletted(_) => paletted_sections += 1,
                }
            }
        }

        let total_sections = chunks.len() * SECTIONS_PER_CHUNK;

        // Calculate theoretical unoptimized size
        // 16x16x64 blocks * 8 bytes per Option<ItemId>
        let unoptimized_per_chunk = 16 * 16 * CHUNK_HEIGHT as usize * 8;
        let unoptimized_total = unoptimized_per_chunk * chunks.len();

        println!("  Generation time: {:?}", gen_time);
        println!(
            "  Memory usage:    {} KB ({:.1}% of unoptimized {} KB)",
            total_memory / 1024,
            (total_memory as f64 / unoptimized_total as f64) * 100.0,
            unoptimized_total / 1024
        );
        println!(
            "  Sections breakdown ({} total, {} per chunk):",
            total_sections, SECTIONS_PER_CHUNK
        );
        println!(
            "    Empty:    {} ({:.1}%)",
            empty_sections,
            (empty_sections as f64 / total_sections as f64) * 100.0
        );
        println!(
            "    Uniform:  {} ({:.1}%)",
            uniform_sections,
            (uniform_sections as f64 / total_sections as f64) * 100.0
        );
        println!(
            "    Paletted: {} ({:.1}%)",
            paletted_sections,
            (paletted_sections as f64 / total_sections as f64) * 100.0
        );
        println!();
    }

    // Memory usage per section type
    println!("--- Section Memory Details ---");
    println!(
        "  CHUNK_HEIGHT: {} blocks ({} sections)",
        CHUNK_HEIGHT, SECTIONS_PER_CHUNK
    );
    println!(
        "  GROUND_LEVEL: {} (surface at y={})",
        GROUND_LEVEL, GROUND_LEVEL
    );
    println!("  SECTION_HEIGHT: {} blocks", SECTION_HEIGHT);

    // Generate a single chunk to analyze
    let chunk = ChunkData::generate(IVec2::ZERO);
    println!("\n--- Single Chunk Analysis (0,0) ---");
    for (i, section) in chunk.sections().iter().enumerate() {
        let y_start = i as i32 * SECTION_HEIGHT;
        let y_end = y_start + SECTION_HEIGHT;
        let section_type = match section {
            ChunkSection::Empty => "Empty",
            ChunkSection::Uniform(_) => "Uniform",
            ChunkSection::Paletted(_) => "Paletted",
        };
        let memory = section.memory_usage();
        println!(
            "  Section {} (y={}..{}): {} ({} bytes)",
            i, y_start, y_end, section_type, memory
        );
    }
    println!("  Total chunk memory: {} bytes", chunk.memory_usage());

    println!("\n=== Benchmark Complete ===");
}
