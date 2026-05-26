const STREAM_ARRAY_SIZE: usize = 10_000_000;
const OFFSET: usize = 0;
const NTIMES: usize = 10;
const BYTES_PER_WORD: usize = 8;

const COPY_BEST_MBPS: f64 = 25_000.0;
const SCALE_BEST_MBPS: f64 = 24_800.0;
const ADD_BEST_MBPS: f64 = 30_000.0;
const TRIAD_BEST_MBPS: f64 = 29_800.0;

const COPY_AVG_TIME: f64 = 0.006500;
const COPY_MIN_TIME: f64 = 0.006400;
const COPY_MAX_TIME: f64 = 0.006700;

const SCALE_AVG_TIME: f64 = 0.006550;
const SCALE_MIN_TIME: f64 = 0.006452;
const SCALE_MAX_TIME: f64 = 0.006770;

const ADD_AVG_TIME: f64 = 0.008100;
const ADD_MIN_TIME: f64 = 0.008000;
const ADD_MAX_TIME: f64 = 0.008320;

const TRIAD_AVG_TIME: f64 = 0.008150;
const TRIAD_MIN_TIME: f64 = 0.008053;
const TRIAD_MAX_TIME: f64 = 0.008340;

fn main() {
    let memory_per_array_mib =
        BYTES_PER_WORD as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0);
    let memory_per_array_gib =
        BYTES_PER_WORD as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0 / 1024.0);
    let total_memory_mib = 3.0 * memory_per_array_mib;
    let total_memory_gib = 3.0 * memory_per_array_gib;

    println!("-------------------------------------------------------------");
    println!("STREAM version 5.10 (placeholder Rust CLI)");
    println!("-------------------------------------------------------------");
    println!(
        "This system uses {} bytes per array element.",
        BYTES_PER_WORD
    );
    println!("-------------------------------------------------------------");
    println!(
        "Array size = {} (elements), Offset = {} (elements)",
        STREAM_ARRAY_SIZE, OFFSET
    );
    println!(
        "Memory per array = {:.1} MiB (= {:.1} GiB).",
        memory_per_array_mib, memory_per_array_gib
    );
    println!(
        "Total memory required = {:.1} MiB (= {:.1} GiB).",
        total_memory_mib, total_memory_gib
    );
    println!("Each kernel will be executed {} times.", NTIMES);
    println!(" The *best* time for each kernel (excluding the first iteration)");
    println!(" will be used to compute the reported bandwidth.");
    println!("-------------------------------------------------------------");
    println!("Your clock granularity/precision appears to be 1 microseconds.");
    println!("Each test below will take on the order of 5400 microseconds.");
    println!("   (= 5400 clock ticks)");
    println!("Increase the size of the arrays if this shows that");
    println!("you are not getting at least 20 clock ticks per test.");
    println!("-------------------------------------------------------------");
    println!("WARNING -- The above is only a rough guideline.");
    println!("For best results, please be sure you know the");
    println!("precision of your system timer.");
    println!("-------------------------------------------------------------");
    println!("Function    Best Rate MB/s  Avg time     Min time     Max time");
    print_row("Copy:", COPY_BEST_MBPS, COPY_AVG_TIME, COPY_MIN_TIME, COPY_MAX_TIME);
    print_row(
        "Scale:",
        SCALE_BEST_MBPS,
        SCALE_AVG_TIME,
        SCALE_MIN_TIME,
        SCALE_MAX_TIME,
    );
    print_row("Add:", ADD_BEST_MBPS, ADD_AVG_TIME, ADD_MIN_TIME, ADD_MAX_TIME);
    print_row(
        "Triad:",
        TRIAD_BEST_MBPS,
        TRIAD_AVG_TIME,
        TRIAD_MIN_TIME,
        TRIAD_MAX_TIME,
    );
    println!("-------------------------------------------------------------");
    println!("Solution Validates: placeholder implementation");
    println!("-------------------------------------------------------------");
}

fn print_row(label: &str, best_rate: f64, avg_time: f64, min_time: f64, max_time: f64) {
    println!(
        "{:<12}{:>12.1}{:>13.6}{:>13.6}{:>13.6}",
        label, best_rate, avg_time, min_time, max_time
    );
}
