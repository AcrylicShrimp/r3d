// Below algorithm is based on the paper: "Distance Transforms of Sampled Functions"
// See: https://cs.brown.edu/people/pfelzens/papers/dt-final.pdf

// Also referenced: https://github.com/mapbox/tiny-sdf/blob/main/index.js,

use fontdue::Metrics;

pub fn generate_sdf(
    metrics: &Metrics,
    rasterized: &[u8],
    inset: usize,
    radius: usize,
    cutoff: f32,
) -> Vec<u8> {
    let width = metrics.width + 2 * inset;
    let height = metrics.height + 2 * inset;
    let len = width * height;
    let mut grid_outer = vec![f32::INFINITY; len];
    let mut grid_inner = vec![0f32; len];

    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let alpha = rasterized[y * metrics.width + x];
            if alpha == 0 {
                continue;
            }

            let index = (y + inset) * width + x + inset;

            if alpha == 255 {
                grid_outer[index] = 0f32;
                grid_inner[index] = f32::INFINITY;
            } else {
                let alpha = (255 - alpha) as f32 / 255f32;
                grid_outer[index] = if 0f32 < alpha { alpha * alpha } else { 0f32 };
                grid_inner[index] = if alpha < 0f32 { alpha * alpha } else { 0f32 };
            }
        }
    }

    let mut v = vec![0usize; len];
    let mut z = vec![0f32; len + 1];
    let mut f = vec![0f32; len];

    s_edt_2d(
        &mut grid_outer,
        width,
        0,
        0,
        width,
        height,
        &mut v,
        &mut z,
        &mut f,
    );
    s_edt_2d(
        &mut grid_inner,
        width,
        inset,
        inset,
        metrics.width,
        metrics.height,
        &mut v,
        &mut z,
        &mut f,
    );

    let radius = radius as f32;
    let mut sdf = vec![0; width * height];

    for index in 0..len {
        let distance = grid_outer[index].sqrt() - grid_inner[index].sqrt();
        sdf[index] = (255f32 - 255f32 * (distance / radius + cutoff)).round() as u8;
    }

    sdf
}

// 2-D squared Euclidean distance transform
pub fn s_edt_2d(
    grid: &mut [f32],
    grid_width: usize,
    offset_x: usize,
    offset_y: usize,
    width: usize,
    height: usize,
    v: &mut [usize],
    z: &mut [f32],
    f: &mut [f32],
) {
    for x in offset_x..(offset_x + width) {
        s_edt_1d(grid, offset_y * grid_width + x, grid_width, height, v, z, f);
    }
    for y in offset_y..(offset_y + height) {
        s_edt_1d(grid, y * grid_width + offset_x, 1, width, v, z, f);
    }
}

// 1-D squared Euclidean distance transform
pub fn s_edt_1d(
    grid: &mut [f32],
    offset: usize,
    stride: usize,
    length: usize,
    v: &mut [usize], // The horizontal grid location of the i-th parabola in the lower envelope is stored in v[i].
    z: &mut [f32], // The range in which the i-th parabola of the lower envelope is below the others is given by z[i] and z[i+1].
    f: &mut [f32],
) {
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    f[0] = grid[offset];

    let mut k = 0; // The variable k keeps track of the number of parabolas in the lower envelope.

    for q in 1..length {
        f[q] = grid[offset + q * stride];

        let mut s;
        let q2 = q * q;

        let is_k_underflow = loop {
            let r = v[k];
            s = (f[q] - f[r] + (q2 - r * r) as f32) / (q - r) as f32 * 0.5f32;

            if z[k] < s {
                break false;
            }

            if k == 0 {
                break true;
            }

            k -= 1;
        };

        if !is_k_underflow {
            k += 1;
        }

        v[k] = q;
        z[k] = s;
        z[k + 1] = f32::INFINITY;
    }

    k = 0;

    for q in 0..length {
        while z[k + 1] < q as f32 {
            k += 1;
        }

        let r = v[k];
        let qr = q as isize - r as isize;
        grid[offset + q * stride] = f[r] + (qr * qr) as f32;
    }
}
