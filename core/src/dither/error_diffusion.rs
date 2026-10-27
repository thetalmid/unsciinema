//! Error diffusion dithering algorithms (CPU-only, sequential)

use crate::dither::DitherAlgorithm;

fn clamp_u8(v: i32) -> u8 { v.clamp(0, 255) as u8 }

/// Floyd-Steinberg (1976) - classic error diffusion
/// Weights: 7/16, 3/16, 5/16, 1/16
pub fn floyd_steinberg(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            if x + 1 < w {
                luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + err * 7 / 16);
            }
            if x > 0 && y + 1 < h {
                luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + err * 3 / 16);
            }
            if y + 1 < h {
                luma[idx + w] = clamp_u8(luma[idx + w] as i32 + err * 5 / 16);
            }
            if x + 1 < w && y + 1 < h {
                luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + err * 1 / 16);
            }
        }
    }
}

/// Atkinson (1984) - Macintosh, distributes 75% of error
pub fn atkinson(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            let e = err / 8;
            if x + 1 < w { luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + e); }
            if x + 2 < w { luma[idx + 2] = clamp_u8(luma[idx + 2] as i32 + e); }
            if x > 0 && y + 1 < h { luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + e); }
            if y + 1 < h { luma[idx + w] = clamp_u8(luma[idx + w] as i32 + e); }
            if x + 1 < w && y + 1 < h { luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + e); }
            if y + 2 < h { luma[idx + 2 * w] = clamp_u8(luma[idx + 2 * w] as i32 + e); }
        }
    }
}

/// Sierra (3-row, lighter)
pub fn sierra(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            if x + 1 < w { luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + err * 5 / 32); }
            if x + 2 < w { luma[idx + 2] = clamp_u8(luma[idx + 2] as i32 + err * 3 / 32); }
            if y + 1 < h {
                if x > 1 { luma[idx + w - 2] = clamp_u8(luma[idx + w - 2] as i32 + err * 2 / 32); }
                if x > 0 { luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + err * 4 / 32); }
                luma[idx + w] = clamp_u8(luma[idx + w] as i32 + err * 5 / 32);
                if x + 1 < w { luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + err * 4 / 32); }
                if x + 2 < w { luma[idx + w + 2] = clamp_u8(luma[idx + w + 2] as i32 + err * 2 / 32); }
            }
            if y + 2 < h {
                if x > 0 { luma[idx + 2*w - 1] = clamp_u8(luma[idx + 2*w - 1] as i32 + err * 2 / 32); }
                luma[idx + 2*w] = clamp_u8(luma[idx + 2*w] as i32 + err * 3 / 32);
                if x + 1 < w { luma[idx + 2*w + 1] = clamp_u8(luma[idx + 2*w + 1] as i32 + err * 2 / 32); }
            }
        }
    }
}

/// Jarvis-Judice-Ninke (3-row, 12 neighbors, smoother)
pub fn jarvis_judice_ninke(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            if x + 1 < w { luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + err * 7 / 48); }
            if x + 2 < w { luma[idx + 2] = clamp_u8(luma[idx + 2] as i32 + err * 5 / 48); }
            if y + 1 < h {
                if x > 1 { luma[idx + w - 2] = clamp_u8(luma[idx + w - 2] as i32 + err * 3 / 48); }
                if x > 0 { luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + err * 5 / 48); }
                luma[idx + w] = clamp_u8(luma[idx + w] as i32 + err * 7 / 48);
                if x + 1 < w { luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + err * 5 / 48); }
                if x + 2 < w { luma[idx + w + 2] = clamp_u8(luma[idx + w + 2] as i32 + err * 3 / 48); }
            }
            if y + 2 < h {
                if x > 1 { luma[idx + 2*w - 2] = clamp_u8(luma[idx + 2*w - 2] as i32 + err * 1 / 48); }
                if x > 0 { luma[idx + 2*w - 1] = clamp_u8(luma[idx + 2*w - 1] as i32 + err * 3 / 48); }
                luma[idx + 2*w] = clamp_u8(luma[idx + 2*w] as i32 + err * 5 / 48);
                if x + 1 < w { luma[idx + 2*w + 1] = clamp_u8(luma[idx + 2*w + 1] as i32 + err * 3 / 48); }
                if x + 2 < w { luma[idx + 2*w + 2] = clamp_u8(luma[idx + 2*w + 2] as i32 + err * 1 / 48); }
            }
        }
    }
}

/// Stucki (3-row, similar to JJN but faster)
pub fn stucki(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            if x + 1 < w { luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + err * 8 / 42); }
            if x + 2 < w { luma[idx + 2] = clamp_u8(luma[idx + 2] as i32 + err * 4 / 42); }
            if y + 1 < h {
                if x > 1 { luma[idx + w - 2] = clamp_u8(luma[idx + w - 2] as i32 + err * 2 / 42); }
                if x > 0 { luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + err * 4 / 42); }
                luma[idx + w] = clamp_u8(luma[idx + w] as i32 + err * 8 / 42);
                if x + 1 < w { luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + err * 4 / 42); }
                if x + 2 < w { luma[idx + w + 2] = clamp_u8(luma[idx + w + 2] as i32 + err * 2 / 42); }
            }
            if y + 2 < h {
                if x > 1 { luma[idx + 2*w - 2] = clamp_u8(luma[idx + 2*w - 2] as i32 + err * 1 / 42); }
                if x > 0 { luma[idx + 2*w - 1] = clamp_u8(luma[idx + 2*w - 1] as i32 + err * 2 / 42); }
                luma[idx + 2*w] = clamp_u8(luma[idx + 2*w] as i32 + err * 4 / 42);
                if x + 1 < w { luma[idx + 2*w + 1] = clamp_u8(luma[idx + 2*w + 1] as i32 + err * 2 / 42); }
                if x + 2 < w { luma[idx + 2*w + 2] = clamp_u8(luma[idx + 2*w + 2] as i32 + err * 1 / 42); }
            }
        }
    }
}

/// Burkes (2-row, lighter version of Stucki)
pub fn burkes(luma: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = luma[idx] as i32;
            let new = if (old as f32 / 255.0) > threshold { 255 } else { 0 };
            luma[idx] = lerp(old, new, strength);
            let err = old - new as i32;
            
            if err == 0 { continue; }
            
            if x + 1 < w { luma[idx + 1] = clamp_u8(luma[idx + 1] as i32 + err * 8 / 32); }
            if x + 2 < w { luma[idx + 2] = clamp_u8(luma[idx + 2] as i32 + err * 4 / 32); }
            if y + 1 < h {
                if x > 1 { luma[idx + w - 2] = clamp_u8(luma[idx + w - 2] as i32 + err * 2 / 32); }
                if x > 0 { luma[idx + w - 1] = clamp_u8(luma[idx + w - 1] as i32 + err * 4 / 32); }
                luma[idx + w] = clamp_u8(luma[idx + w] as i32 + err * 8 / 32);
                if x + 1 < w { luma[idx + w + 1] = clamp_u8(luma[idx + w + 1] as i32 + err * 4 / 32); }
                if x + 2 < w { luma[idx + w + 2] = clamp_u8(luma[idx + w + 2] as i32 + err * 2 / 32); }
            }
        }
    }
}

fn lerp(old: i32, new: i32, strength: f32) -> u8 {
    ((old as f32 * (1.0 - strength) + new as f32 * strength).round() as i32).clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floyd_steinberg_basic() {
        let mut luma = vec![128; 16]; // 4x4 mid-gray
        floyd_steinberg(&mut luma, 4, 4, 1.0, 0.5);
        let white = luma.iter().filter(|&&v| v == 255).count();
        let black = luma.iter().filter(|&&v| v == 0).count();
        assert!(white > 0 && black > 0);
        assert_eq!(white + black, 16);
    }

    #[test]
    fn test_floyd_steinberg_strength_zero() {
        let mut luma = vec![128; 16];
        let original = luma.clone();
        floyd_steinberg(&mut luma, 4, 4, 0.0, 0.5);
        assert_eq!(luma, original);
    }

    #[test]
    fn test_floyd_steinberg_all_white() {
        let mut luma = vec![255; 16];
        floyd_steinberg(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 255));
    }

    #[test]
    fn test_floyd_steinberg_all_black() {
        let mut luma = vec![0; 16];
        floyd_steinberg(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_atkinson_basic() {
        let mut luma = vec![128; 16];
        atkinson(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_atkinson_75_percent_error() {
        let mut luma = vec![128; 16];
        atkinson(&mut luma, 4, 4, 1.0, 0.5);
        let white = luma.iter().filter(|&&v| v == 255).count();
        let black = luma.iter().filter(|&&v| v == 0).count();
        assert!(white > 0 && black > 0);
        assert_eq!(white + black, 16);
    }

    #[test]
    fn test_sierra_basic() {
        let mut luma = vec![128; 16];
        sierra(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_jarvis_judice_ninke_basic() {
        let mut luma = vec![128; 16];
        jarvis_judice_ninke(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_stucki_basic() {
        let mut luma = vec![128; 16];
        stucki(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_burkes_basic() {
        let mut luma = vec![128; 16];
        burkes(&mut luma, 4, 4, 1.0, 0.5);
        assert!(luma.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(100, 200, 0.0), 100);
        assert_eq!(lerp(100, 200, 1.0), 200);
        assert_eq!(lerp(100, 200, 0.5), 150);
        assert_eq!(lerp(0, 255, 0.5), 128);
    }

    #[test]
    fn test_lerp_clamp() {
        assert_eq!(lerp(0, 255, 2.0), 255);
        assert_eq!(lerp(255, 0, 2.0), 0);
    }
}