//! Helper functions for colors

/// Blend two color slices by a certain amount.
pub fn blend(a: &[u8], b: &[u8], blend: f32, out: &mut [u8]) {
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
            let v = (*a as f32 + (*b as f32 - *a as f32) * blend).round() as u8;
            out[i] = v;
        }
    }
}

/// Blend color b into color a.
pub fn blend_into(a: &mut [u8], b: &[u8], blend: f32) {
    if blend <= 0.0 {
    } else if blend >= 1.0 {
        for (i, b) in b.iter().enumerate() {
            a[i] = *b;
        }
    } else {
        for (a, b) in a.iter_mut().zip(b) {
            let aa = *a;
            *a = (aa as f32 + (*b as f32 - aa as f32) * blend).round() as u8;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::color::{blend, blend_into};

    #[test]
    fn blend_test() {
        let a = [0; 4];
        let b = [255; 4];
        let mut c = [0; 4];

        blend(&a, &b, 0.0, &mut c);
        assert_eq!(c, [0; 4]);

        blend(&a, &b, 0.5, &mut c);
        assert_eq!(c, [128; 4]);

        blend(&a, &b, 1.0, &mut c);
        assert_eq!(c, [255; 4]);
    }
}
