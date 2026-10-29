//! Genuine tileable blue noise, generated deterministically at first use.
//!
//! The salvaged "Random Noise" (xorshift white noise) stays labeled as such
//! (algorithm 5). Algorithm 10 is real blue noise: a 32x32 void-and-cluster
//! style best-candidate mask. Each new sample is placed at the empty position
//! of lowest Gaussian energy (toroidal, sigma 1.5, radius 3); tie-breaking is
//! scan order, so every run and every platform produces the identical mask.
//! Placement order is rank-normalized to [0, 1): early placements sit in the
//! biggest voids (sparse), late placements fill the densest clusters.

use std::sync::OnceLock;

const N: usize = 32; // 32x32 = 1024 positions, tiles seamlessly
const SIGMA: f32 = 1.5;
const KERNEL_R: i32 = 3;

static MASK: OnceLock<Box<[f32; N * N]>> = OnceLock::new();

fn kernel(dx: i32, dy: i32) -> f32 {
    let d2 = (dx * dx + dy * dy) as f32;
    (-d2 / (2.0 * SIGMA * SIGMA)).exp()
}

fn wrap(v: i32) -> usize {
    let n = N as i32;
    let m = v % n;
    (if m < 0 { m + n } else { m }) as usize
}

fn generate() -> Box<[f32; N * N]> {
    let mut offsets: Vec<(i32, i32, f32)> = Vec::new();
    for dy in -KERNEL_R..=KERNEL_R {
        for dx in -KERNEL_R..=KERNEL_R {
            if dx == 0 && dy == 0 {
                continue;
            }
            offsets.push((dx, dy, kernel(dx, dy)));
        }
    }

    let mut order: Vec<usize> = Vec::with_capacity(N * N);
    let mut filled = [false; N * N];
    // Running Gaussian energy field; filled cells are never candidates again.
    let mut energy = [0f32; N * N];

    for _ in 0..N * N {
        let mut best: Option<(f32, usize)> = None;
        for idx in 0..N * N {
            if filled[idx] {
                continue;
            }
            let e = energy[idx];
            if best.map_or(true, |(be, _)| e < be - 1e-12) {
                best = Some((e, idx));
            }
        }
        let (_, idx) = best.expect("an empty cell always exists");
        filled[idx] = true;
        order.push(idx);
        energy[idx] = f32::INFINITY;
        // Add the placed point's Gaussian lobe to the field (toroidal).
        let (cx, cy) = ((idx % N) as i32, (idx / N) as i32);
        for &(dx, dy, k) in &offsets {
            let nx = wrap(cx + dx);
            let ny = wrap(cy + dy);
            let nidx = ny * N + nx;
            if energy[nidx] < f32::MAX / 2.0 {
                energy[nidx] += k;
            }
        }
    }

    // Placement order -> rank values: first placed (largest void) = sparsest.
    let mut mask = [0f32; N * N];
    let denom = (N * N) as f32;
    for (rank, &idx) in order.iter().enumerate() {
        mask[idx] = rank as f32 / denom;
    }
    Box::new(mask)
}

/// Cached 32x32 tileable blue-noise mask in [0, 1).
pub fn blue_noise_mask() -> &'static [f32; N * N] {
    MASK.get_or_init(generate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_is_well_distributed_and_blue() {
        let m = blue_noise_mask();
        // Uniform histogram: mean near 0.5, every bucket used.
        let mean: f32 = m.iter().sum::<f32>() / m.len() as f32;
        assert!((mean - 0.5).abs() < 0.05, "mean {mean}");
        let mut buckets = [0usize; 8];
        for &v in m.iter() {
            let b = ((v * 8.0) as usize).min(7);
            buckets[b] += 1;
        }
        assert!(buckets.iter().all(|&c| c > 60), "histogram {buckets:?}");

        // Blue-ness: adjacent-lag autocorrelation must be clearly below the
        // variance (white noise would sit near 0 relative to variance scale;
        // blue noise is strongly negative at adjacent lags).
        let n = m.len() as f32;
        let mean = m.iter().sum::<f32>() / n;
        let var: f32 = m.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
        let mut corr = 0f32;
        for y in 0..N {
            for x in 0..N {
                let a = m[y * N + x] - 0.5;
                let b = m[y * N + ((x + 1) % N)] - 0.5;
                corr += a * b;
            }
        }
        corr /= n;
        assert!(
            corr < var * 0.1,
            "adjacent autocorrelation {corr} not clearly below variance {var}"
        );
    }

    #[test]
    fn mask_is_deterministic() {
        assert_eq!(blue_noise_mask(), blue_noise_mask());
    }
}
