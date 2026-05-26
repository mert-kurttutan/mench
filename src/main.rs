use std::env;
use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

const STREAM_ARRAY_SIZE: usize = 10_000_000;
const OFFSET: usize = 0;
const NTIMES: usize = 10;
const BYTES_PER_WORD: usize = std::mem::size_of::<f64>();
const SCALAR: f64 = 3.0;
const HLINE: &str = "-------------------------------------------------------------";

fn main() {
    let thread_count = resolve_thread_count();
    let mut a = vec![1.0_f64; STREAM_ARRAY_SIZE + OFFSET];
    let mut b = vec![2.0_f64; STREAM_ARRAY_SIZE + OFFSET];
    let mut c = vec![0.0_f64; STREAM_ARRAY_SIZE + OFFSET];
    let mut times = [[0.0_f64; NTIMES]; 4];

    a.iter_mut().for_each(|value| *value *= 2.0);

    let memory_per_array_mib =
        BYTES_PER_WORD as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0);
    let memory_per_array_gib =
        BYTES_PER_WORD as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0 / 1024.0);
    let total_memory_mib = 3.0 * memory_per_array_mib;
    let total_memory_gib = 3.0 * memory_per_array_gib;
    let quantum = check_tick();

    println!("{HLINE}");
    println!("STREAM version 5.10 (Rust)");
    println!("{HLINE}");
    println!("This system uses {BYTES_PER_WORD} bytes per array element.");
    println!("{HLINE}");
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
    println!("Each kernel will be executed {NTIMES} times.");
    println!(" The *best* time for each kernel (excluding the first iteration)");
    println!(" will be used to compute the reported bandwidth.");
    println!("Number of Threads requested = {thread_count}");
    println!("{HLINE}");
    println!(
        "Your clock granularity/precision appears to be {} microseconds.",
        quantum
    );

    let estimate = estimate_test_time(&a, &mut c, thread_count);
    let ticks = (estimate * 1_000_000.0) / quantum as f64;
    println!(
        "Each test below will take on the order of {:.0} microseconds.",
        estimate * 1_000_000.0
    );
    println!("   (= {:.0} clock ticks)", ticks);
    println!("Increase the size of the arrays if this shows that");
    println!("you are not getting at least 20 clock ticks per test.");
    println!("{HLINE}");
    println!("WARNING -- The above is only a rough guideline.");
    println!("For best results, please be sure you know the");
    println!("precision of your system timer.");
    println!("{HLINE}");

    for k in 0..NTIMES {
        let start = Instant::now();
        copy_kernel(&a, &mut c, thread_count);
        times[0][k] = start.elapsed().as_secs_f64();

        let start = Instant::now();
        scale_kernel(&c, &mut b, SCALAR, thread_count);
        times[1][k] = start.elapsed().as_secs_f64();

        let start = Instant::now();
        add_kernel(&a, &b, &mut c, thread_count);
        times[2][k] = start.elapsed().as_secs_f64();

        let start = Instant::now();
        triad_kernel(&b, &c, &mut a, SCALAR, thread_count);
        times[3][k] = start.elapsed().as_secs_f64();
    }

    let bytes = [
        2.0 * BYTES_PER_WORD as f64 * STREAM_ARRAY_SIZE as f64,
        2.0 * BYTES_PER_WORD as f64 * STREAM_ARRAY_SIZE as f64,
        3.0 * BYTES_PER_WORD as f64 * STREAM_ARRAY_SIZE as f64,
        3.0 * BYTES_PER_WORD as f64 * STREAM_ARRAY_SIZE as f64,
    ];
    let labels = ["Copy:", "Scale:", "Add:", "Triad:"];

    println!("Function    Best Rate MB/s  Avg time     Min time     Max time");
    for i in 0..4 {
        let mut avg = 0.0;
        let mut min = f64::MAX;
        let mut max = 0.0;

        for &t in &times[i][1..] {
            avg += t;
            if t < min {
                min = t;
            }
            if t > max {
                max = t;
            }
        }

        avg /= (NTIMES - 1) as f64;
        let rate = 1.0e-6 * bytes[i] / min;
        print_row(labels[i], rate, avg, min, max);
    }

    println!("{HLINE}");
    validate_results(&a, &b, &c);
    println!("{HLINE}");
}

fn copy_kernel(a: &[f64], c: &mut [f64], thread_count: usize) {
    let chunk_len = chunk_len(thread_count);
    thread::scope(|scope| {
        for (src, dst) in a[..STREAM_ARRAY_SIZE]
            .chunks(chunk_len)
            .zip(c[..STREAM_ARRAY_SIZE].chunks_mut(chunk_len))
        {
            scope.spawn(move || {
                dst.copy_from_slice(src);
            });
        }
    });
    black_box(&*c);
}

fn scale_kernel(c: &[f64], b: &mut [f64], scalar: f64, thread_count: usize) {
    let chunk_len = chunk_len(thread_count);
    thread::scope(|scope| {
        for (src, dst) in c[..STREAM_ARRAY_SIZE]
            .chunks(chunk_len)
            .zip(b[..STREAM_ARRAY_SIZE].chunks_mut(chunk_len))
        {
            scope.spawn(move || {
                for i in 0..dst.len() {
                    dst[i] = scalar * src[i];
                }
            });
        }
    });
    black_box(&*b);
}

fn add_kernel(a: &[f64], b: &[f64], c: &mut [f64], thread_count: usize) {
    let chunk_len = chunk_len(thread_count);
    thread::scope(|scope| {
        for (index, dst) in c[..STREAM_ARRAY_SIZE].chunks_mut(chunk_len).enumerate() {
            let start = index * chunk_len;
            let a_src = &a[start..start + dst.len()];
            let b_src = &b[start..start + dst.len()];
            scope.spawn(move || {
                for i in 0..dst.len() {
                    dst[i] = a_src[i] + b_src[i];
                }
            });
        }
    });
    black_box(&*c);
}

fn triad_kernel(b: &[f64], c: &[f64], a: &mut [f64], scalar: f64, thread_count: usize) {
    let chunk_len = chunk_len(thread_count);
    thread::scope(|scope| {
        for (index, dst) in a[..STREAM_ARRAY_SIZE].chunks_mut(chunk_len).enumerate() {
            let start = index * chunk_len;
            let b_src = &b[start..start + dst.len()];
            let c_src = &c[start..start + dst.len()];
            scope.spawn(move || {
                for i in 0..dst.len() {
                    dst[i] = b_src[i] + scalar * c_src[i];
                }
            });
        }
    });
    black_box(&*a);
}

fn estimate_test_time(a: &[f64], c: &mut [f64], thread_count: usize) -> f64 {
    let start = Instant::now();
    copy_kernel(a, c, thread_count);
    start.elapsed().as_secs_f64()
}

fn check_tick() -> u64 {
    let mut min_delta = Duration::from_secs(1);
    let mut last = Instant::now();

    for _ in 0..1_000 {
        let mut current = Instant::now();
        while current == last {
            current = Instant::now();
        }

        let delta = current.duration_since(last);
        if delta < min_delta {
            min_delta = delta;
        }
        last = current;
    }

    let micros = min_delta.as_micros() as u64;
    micros.max(1)
}

fn validate_results(a: &[f64], b: &[f64], c: &[f64]) {
    let mut aj = 1.0_f64;
    let mut bj = 2.0_f64;
    let mut cj = 0.0_f64;

    aj *= 2.0;
    for _ in 0..NTIMES {
        cj = aj;
        bj = SCALAR * cj;
        cj = aj + bj;
        aj = bj + SCALAR * cj;
    }

    let mut asum = 0.0;
    let mut bsum = 0.0;
    let mut csum = 0.0;
    for i in 0..STREAM_ARRAY_SIZE {
        asum += (a[i] - aj).abs();
        bsum += (b[i] - bj).abs();
        csum += (c[i] - cj).abs();
    }

    let aavg = asum / STREAM_ARRAY_SIZE as f64 / aj.abs();
    let bavg = bsum / STREAM_ARRAY_SIZE as f64 / bj.abs();
    let cavg = csum / STREAM_ARRAY_SIZE as f64 / cj.abs();

    if aavg < 1.0e-13 && bavg < 1.0e-13 && cavg < 1.0e-13 {
        println!("Solution Validates: avg error less than 1.000000e-13 on all three arrays");
    } else {
        println!(
            "Validation failed: avg errors are {:.6e}, {:.6e}, {:.6e}",
            aavg, bavg, cavg
        );
    }
}

fn print_row(label: &str, best_rate: f64, avg_time: f64, min_time: f64, max_time: f64) {
    println!(
        "{:<12}{:>12.1}{:>13.6}{:>13.6}{:>13.6}",
        label, best_rate, avg_time, min_time, max_time
    );
}

fn resolve_thread_count() -> usize {
    let default = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let from_arg = parse_threads_arg();
    let from_env = env::var("MENCH_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());

    from_arg
        .or(from_env)
        .filter(|&count| count > 0)
        .unwrap_or(default)
        .min(STREAM_ARRAY_SIZE.max(1))
}

fn parse_threads_arg() -> Option<usize> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--threads" {
            return args.next().and_then(|value| value.parse::<usize>().ok());
        }

        if let Some(value) = arg.strip_prefix("--threads=") {
            return value.parse::<usize>().ok();
        }
    }

    None
}

fn chunk_len(thread_count: usize) -> usize {
    STREAM_ARRAY_SIZE.div_ceil(thread_count.max(1))
}
