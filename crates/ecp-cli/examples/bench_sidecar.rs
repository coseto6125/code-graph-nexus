use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn sidecar_path(graph_path: &Path) -> PathBuf {
    let mut p = graph_path.as_os_str().to_owned();
    p.push(".compatible_version");
    PathBuf::from(p)
}

fn read_sidecar(graph_path: &Path) -> Option<u32> {
    let content = fs::read_to_string(sidecar_path(graph_path)).ok()?;
    content.trim().parse::<u32>().ok()
}

fn bench<F: FnMut()>(label: &str, n: usize, mut f: F) {
    let warmup = 5;
    for _ in 0..warmup {
        f();
    }
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    let dur = t.elapsed();
    let per_call = dur / n as u32;
    println!(
        "{:<40} n={n} total={:>10.3?} avg/call={:>10.3?}",
        label, dur, per_call
    );
}

fn main() {
    let graph_bin = std::env::args()
        .nth(1)
        .expect("usage: bench_sidecar <path-to-graph.bin>");
    let graph_path = PathBuf::from(&graph_bin);
    assert!(graph_path.is_file(), "graph.bin not found: {graph_bin}");
    let size_mb = fs::metadata(&graph_path).unwrap().len() as f64 / 1_048_576.0;
    println!("graph.bin size: {size_mb:.1} MB ({graph_bin})");

    let scp = sidecar_path(&graph_path);
    fs::write(&scp, b"10\n").expect("write sidecar");
    println!("sidecar at: {}", scp.display());

    let n = 100;
    println!("\n=== {n}-call bench, 5 warmup ===");

    bench("sidecar read (4-byte file)", n, || {
        let v = read_sidecar(&graph_path);
        std::hint::black_box(v);
    });

    bench("header_compatible (mmap+rkyv::access)", n, || {
        let ok = ecp_cli::engine::header_compatible(&graph_path);
        std::hint::black_box(ok);
    });

    println!("\n=== cold-cache attempt (drop pagecache between runs would need root; warm-cache only here) ===");
}
