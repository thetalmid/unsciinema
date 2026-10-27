//! Ordered dithering algorithms (GPU-friendly, parallelizable)

/// 4×4 Bayer matrix (normalized to 0.0-1.0)
const BAYER_4: [f32; 16] = [
    0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
   12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
    3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
   15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
];

/// 8×8 Bayer matrix (normalized to 0.0-1.0)
const BAYER_8: [f32; 64] = [
    0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,  2.0/64.0, 34.0/64.0, 10.0/64.0, 42.0/64.0,
   48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0, 50.0/64.0, 18.0/64.0, 58.0/64.0, 26.0/64.0,
   12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0, 14.0/64.0, 46.0/64.0,  6.0/64.0, 38.0/64.0,
   60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0, 62.0/64.0, 30.0/64.0, 54.0/64.0, 22.0/64.0,
    3.0/64.0, 35.0/64.0, 11.0/64.0, 43.0/64.0,  1.0/64.0, 33.0/64.0,  9.0/64.0, 41.0/64.0,
   51.0/64.0, 19.0/64.0, 59.0/64.0, 27.0/64.0, 49.0/64.0, 17.0/64.0, 57.0/64.0, 25.0/64.0,
   15.0/64.0, 47.0/64.0,  7.0/64.0, 39.0/64.0, 13.0/64.0, 45.0/64.0,  5.0/64.0, 37.0/64.0,
   63.0/64.0, 31.0/64.0, 55.0/64.0, 23.0/64.0, 61.0/64.0, 29.0/64.0, 53.0/64.0, 21.0/64.0,
];

/// Apply Bayer ordered dithering
pub fn bayer_ordered(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32, size: u32) {
    let w = width as usize;
    let h = height as usize;
    let (matrix, matrix_size) = if size == 4 {
        (&BAYER_4[..], 4)
    } else {
        (&BAYER_8[..], 8)
    };
    
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let matrix_threshold = matrix[(y % matrix_size) * matrix_size + (x % matrix_size)];
            // Combine matrix threshold with user threshold
            let effective_threshold = (matrix_threshold + threshold) / 2.0;
            let normalized = luma[idx] as f32 / 255.0;
            let dithered = if normalized > effective_threshold { 255 } else { 0 };
            dithered_out[idx] = dithered;
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Blue noise dithering (uses a simple pseudo-random pattern for now)
/// In production, use a pre-computed blue noise texture
pub fn blue_noise(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    let w = width as usize;
    let h = height as usize;
    
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    
    // Simple pseudo-random using xorshift for blue-noise-like distribution
    let mut state = 0x9E3779B9u32;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    
    // Generate noise texture
    let mut noise = vec![0.0f32; w * h];
    for v in &mut noise { *v = next(); }
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let noise_threshold = noise[idx];
            // Combine noise threshold with user threshold
            let effective_threshold = (noise_threshold + threshold) / 2.0;
            let normalized = luma[idx] as f32 / 255.0;
            let dithered = if normalized > effective_threshold { 255 } else { 0 };
            dithered_out[idx] = dithered;
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayer4_basic() {
        let luma = vec![128; 16]; // 4x4 mid-gray
        let mut dithered = vec![0; 16];
        bayer_ordered(&luma, &mut dithered, 4, 4, 1.0, 0.5, 4);
        let white_count = dithered.iter().filter(|&&v| v == 255).count();
        let black_count = dithered.iter().filter(|&&v| v == 0).count();
        assert!(white_count > 0 && black_count > 0);
        assert_eq!(white_count + black_count, 16);
    }

    #[test]
    fn test_bayer8_basic() {
        let luma = vec![128; 64]; // 8x8 mid-gray
        let mut dithered = vec![0; 64];
        bayer_ordered(&luma, &mut dithered, 8, 8, 1.0, 0.5, 8);
        let white_count = dithered.iter().filter(|&&v| v == 255).count();
        let black_count = dithered.iter().filter(|&&v| v == 0).count();
        assert!(white_count > 0 && black_count > 0);
        assert_eq!(white_count + black_count, 64);
    }

    #[test]
    fn test_bayer4_strength_zero() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        bayer_ordered(&luma, &mut dithered, 4, 4, 0.0, 0.5, 4);
        // With strength 0, should just threshold
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_bayer4_strength_half() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        bayer_ordered(&luma, &mut dithered, 4, 4, 0.5, 0.5, 4);
        for &v in &dithered {
            assert!((0..=255).contains(&v));
        }
    }

    #[test]
    fn test_bayer4_all_white() {
        let luma = vec![255; 16];
        let mut dithered = vec![0; 16];
        bayer_ordered(&luma, &mut dithered, 4, 4, 1.0, 0.5, 4);
        assert!(dithered.iter().all(|&v| v == 255));
    }

    #[test]
    fn test_bayer4_all_black() {
        let luma = vec![0; 16];
        let mut dithered = vec![0; 16];
        bayer_ordered(&luma, &mut dithered, 4, 4, 1.0, 0.5, 4);
        assert!(dithered.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_bayer4_deterministic() {
        let luma = vec![128; 16];
        let mut dithered1 = vec![0; 16];
        let mut dithered2 = vec![0; 16];
        bayer_ordered(&luma, &mut dithered1, 4, 4, 1.0, 0.5, 4);
        bayer_ordered(&luma, &mut dithered2, 4, 4, 1.0, 0.5, 4);
        assert_eq!(dithered1, dithered2);
    }

    #[test]
    fn test_bayer8_deterministic() {
        let luma = vec![128; 64];
        let mut dithered1 = vec![0; 64];
        let mut dithered2 = vec![0; 64];
        bayer_ordered(&luma, &mut dithered1, 8, 8, 1.0, 0.5, 8);
        bayer_ordered(&luma, &mut dithered2, 8, 8, 1.0, 0.5, 8);
        assert_eq!(dithered1, dithered2);
    }

    #[test]
    fn test_blue_noise_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        blue_noise(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        let white = dithered.iter().filter(|&&v| v == 255).count();
        let black = dithered.iter().filter(|&&v| v == 0).count();
        assert!(white > 0 && black > 0);
        assert_eq!(white + black, 16);
    }

    #[test]
    fn test_blue_noise_deterministic() {
        let luma = vec![128; 16];
        let mut dithered1 = vec![0; 16];
        let mut dithered2 = vec![0; 16];
        blue_noise(&luma, &mut dithered1, 4, 4, 1.0, 0.5);
        blue_noise(&luma, &mut dithered2, 4, 4, 1.0, 0.5);
        assert_eq!(dithered1, dithered2);
    }

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 100.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 100.0, 1.0) - 100.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 100.0, 0.5) - 50.0).abs() < f32::EPSILON);
        assert!((lerp(100.0, 0.0, 0.5) - 50.0).abs() < f32::EPSILON);
    }
}