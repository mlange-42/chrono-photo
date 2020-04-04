//! Helper functions for colors

/// Blend two color slices by a certain amount.
fn blend(a: &[u8], b: &[u8], blend: f32, out: &mut [u8]) {
    if blend <= 0.0 {
        for (i, a) in a.iter().enumerate() {
            out[i] = *a;
        }
    } else if blend >= 1.0 {
        for (i, b) in b.iter().enumerate() {
            out[i] = *b;
        }
    } else {
        for (i, (a, b)) in a.iter().zip(b).enumerate() {
            let v = (a as f32 + (b - a) as f32 * blend).round() as u8;
            out[i] = v;
        }
    }
}
