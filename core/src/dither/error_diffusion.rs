//! Error diffusion dithering algorithms (CPU-only, sequential)

/// Floyd-Steinberg (1976) - classic error diffusion
/// Weights: 7/16, 3/16, 5/16, 1/16
pub fn floyd_steinberg(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        // Just threshold
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    // Working buffer for error diffusion (f32 for precision)
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            if x + 1 < w {
                work[idx + 1] += err * 7.0 / 16.0;
            }
            if x > 0 && y + 1 < h {
                work[idx + w - 1] += err * 3.0 / 16.0;
            }
            if y + 1 < h {
                work[idx + w] += err * 5.0 / 16.0;
            }
            if x + 1 < w && y + 1 < h {
                work[idx + w + 1] += err * 1.0 / 16.0;
            }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Atkinson (1984) - Macintosh, distributes 75% of error
pub fn atkinson(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            let e = err / 8.0;
            if x + 1 < w { work[idx + 1] += e; }
            if x + 2 < w { work[idx + 2] += e; }
            if x > 0 && y + 1 < h { work[idx + w - 1] += e; }
            if y + 1 < h { work[idx + w] += e; }
            if x + 1 < w && y + 1 < h { work[idx + w + 1] += e; }
            if y + 2 < h { work[idx + 2 * w] += e; }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Sierra (3-row, lighter)
pub fn sierra(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            if x + 1 < w { work[idx + 1] += err * 5.0 / 32.0; }
            if x + 2 < w { work[idx + 2] += err * 3.0 / 32.0; }
            if y + 1 < h {
                if x > 1 { work[idx + w - 2] += err * 2.0 / 32.0; }
                if x > 0 { work[idx + w - 1] += err * 4.0 / 32.0; }
                work[idx + w] += err * 5.0 / 32.0;
                if x + 1 < w { work[idx + w + 1] += err * 4.0 / 32.0; }
                if x + 2 < w { work[idx + w + 2] += err * 2.0 / 32.0; }
            }
            if y + 2 < h {
                if x > 0 { work[idx + 2*w - 1] += err * 2.0 / 32.0; }
                work[idx + 2*w] += err * 3.0 / 32.0;
                if x + 1 < w { work[idx + 2*w + 1] += err * 2.0 / 32.0; }
            }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Jarvis-Judice-Ninke (3-row, 12 neighbors, smoother)
pub fn jarvis_judice_ninke(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            if x + 1 < w { work[idx + 1] += err * 7.0 / 48.0; }
            if x + 2 < w { work[idx + 2] += err * 5.0 / 48.0; }
            if y + 1 < h {
                if x > 1 { work[idx + w - 2] += err * 3.0 / 48.0; }
                if x > 0 { work[idx + w - 1] += err * 5.0 / 48.0; }
                work[idx + w] += err * 7.0 / 48.0;
                if x + 1 < w { work[idx + w + 1] += err * 5.0 / 48.0; }
                if x + 2 < w { work[idx + w + 2] += err * 3.0 / 48.0; }
            }
            if y + 2 < h {
                if x > 1 { work[idx + 2*w - 2] += err * 1.0 / 48.0; }
                if x > 0 { work[idx + 2*w - 1] += err * 3.0 / 48.0; }
                work[idx + 2*w] += err * 5.0 / 48.0;
                if x + 1 < w { work[idx + 2*w + 1] += err * 3.0 / 48.0; }
                if x + 2 < w { work[idx + 2*w + 2] += err * 1.0 / 48.0; }
            }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Stucki (3-row, similar to JJN but faster)
pub fn stucki(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            if x + 1 < w { work[idx + 1] += err * 8.0 / 42.0; }
            if x + 2 < w { work[idx + 2] += err * 4.0 / 42.0; }
            if y + 1 < h {
                if x > 1 { work[idx + w - 2] += err * 2.0 / 42.0; }
                if x > 0 { work[idx + w - 1] += err * 4.0 / 42.0; }
                work[idx + w] += err * 8.0 / 42.0;
                if x + 1 < w { work[idx + w + 1] += err * 4.0 / 42.0; }
                if x + 2 < w { work[idx + w + 2] += err * 2.0 / 42.0; }
            }
            if y + 2 < h {
                if x > 1 { work[idx + 2*w - 2] += err * 1.0 / 42.0; }
                if x > 0 { work[idx + 2*w - 1] += err * 2.0 / 42.0; }
                work[idx + 2*w] += err * 4.0 / 42.0;
                if x + 1 < w { work[idx + 2*w + 1] += err * 2.0 / 42.0; }
                if x + 2 < w { work[idx + 2*w + 2] += err * 1.0 / 42.0; }
            }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

/// Burkes (2-row, lighter version of Stucki)
pub fn burkes(luma: &[u8], dithered_out: &mut [u8], width: u32, height: u32, strength: f32, threshold: f32) {
    if strength <= 0.0 {
        let thresh_val = (threshold * 255.0) as u8;
        for i in 0..luma.len() {
            dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
        }
        return;
    }
    let w = width as usize;
    let h = height as usize;
    
    let mut work: Vec<f32> = luma.iter().map(|&v| v as f32).collect();
    
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = work[idx];
            let new = if old / 255.0 > threshold { 255.0 } else { 0.0 };
            dithered_out[idx] = new as u8;
            let err = old - new;
            
            if err == 0.0 { continue; }
            
            if x + 1 < w { work[idx + 1] += err * 8.0 / 32.0; }
            if x + 2 < w { work[idx + 2] += err * 4.0 / 32.0; }
            if y + 1 < h {
                if x > 1 { work[idx + w - 2] += err * 2.0 / 32.0; }
                if x > 0 { work[idx + w - 1] += err * 4.0 / 32.0; }
                work[idx + w] += err * 8.0 / 32.0;
                if x + 1 < w { work[idx + w + 1] += err * 4.0 / 32.0; }
                if x + 2 < w { work[idx + w + 2] += err * 2.0 / 32.0; }
            }
        }
    }
    
    // Apply strength: blend between original luma and dithered
    if strength < 1.0 {
        for i in 0..luma.len() {
            let dithered = dithered_out[i] as u32;
            let original = luma[i] as u32;
            let blended = ((original as f32 * (1.0 - strength) + dithered as f32 * strength).round() as u32).min(255);
            dithered_out[i] = blended as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floyd_steinberg_basic() {
        let luma = vec![128; 16]; // 4x4 mid-gray
        let mut dithered = vec![0; 16];
        floyd_steinberg(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        let white = dithered.iter().filter(|&&v| v == 255).count();
        let black = dithered.iter().filter(|&&v| v == 0).count();
        assert!(white > 0 && black > 0);
        assert_eq!(white + black, 16);
    }

    #[test]
    fn test_floyd_steinberg_strength_zero() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        floyd_steinberg(&luma, &mut dithered, 4, 4, 0.0, 0.5);
        // With strength 0, should just threshold
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_floyd_steinberg_all_white() {
        let luma = vec![255; 16];
        let mut dithered = vec![0; 16];
        floyd_steinberg(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 255));
    }

    #[test]
    fn test_floyd_steinberg_all_black() {
        let luma = vec![0; 16];
        let mut dithered = vec![0; 16];
        floyd_steinberg(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_atkinson_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        atkinson(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_atkinson_75_percent_error() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        atkinson(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        let white = dithered.iter().filter(|&&v| v == 255).count();
        let black = dithered.iter().filter(|&&v| v == 0).count();
        assert!(white > 0 && black > 0);
        assert_eq!(white + black, 16);
    }

    #[test]
    fn test_sierra_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        sierra(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_jarvis_judice_ninke_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        jarvis_judice_ninke(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_stucki_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        stucki(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn test_burkes_basic() {
        let luma = vec![128; 16];
        let mut dithered = vec![0; 16];
        burkes(&luma, &mut dithered, 4, 4, 1.0, 0.5);
        assert!(dithered.iter().all(|&v| v == 0 || v == 255));
    }

    

    
}