#[cfg(feature = "utils")]

use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use time;

static DEVICE_ID_COUNTER: AtomicUsize  = AtomicUsize::new(0);

// Generates a unique identifier for any VRDisplay
#[allow(dead_code)]
pub fn new_id() -> u32 {
    DEVICE_ID_COUNTER.fetch_add(1, SeqCst) as u32
}

// Returns the current time in milliseconds
#[allow(dead_code)]
pub fn timestamp() -> f64 {
    let timespec = time::get_time();
    timespec.sec as f64 * 1000.0 + (timespec.nsec as f64 * 1e-6)
}

// Multiply 4x4 matrices
#[allow(dead_code)]
pub fn multiply_matrix(a: &[f32; 16], b: &[f32; 16], out: &mut [f32; 16]) {
    let mut tmp: [f32; 16] = unsafe { mem::uninitialized() };

    tmp[0] = b[0] * a[0] + b[1] * a[4] + b[2] * a[8] + b[3] * a[12];
    tmp[1] = b[0] * a[1] + b[1] * a[5] + b[2] * a[9] + b[3] * a[13];
    tmp[2] = b[0] * a[2] + b[1] * a[6] + b[2] * a[10] + b[3] * a[14];
    tmp[3] = b[0] * a[3] + b[1] * a[7] + b[2] * a[11] + b[3] * a[15];
    
    tmp[4] = b[4] * a[0] + b[5] * a[4] + b[6] * a[8] + b[7] * a[12];
    tmp[5] = b[4] * a[1] + b[5] * a[5] + b[6] * a[9] + b[7] * a[13];
    tmp[6] = b[4] * a[2] + b[5] * a[6] + b[6] * a[10] + b[7] * a[14];
    tmp[7] = b[4] * a[3] + b[5] * a[7] + b[6] * a[11] + b[7] * a[15];
    
    tmp[8] = b[8] * a[0] + b[9] * a[4] + b[10] * a[8] + b[11] * a[12];
    tmp[9] = b[8] * a[1] + b[9] * a[5] + b[10] * a[9] + b[11] * a[13];
    tmp[10] = b[8] * a[2] + b[9] * a[6] + b[10] * a[10] + b[11] * a[14];
    tmp[11] = b[8] * a[3] + b[9] * a[7] + b[10] * a[11] + b[11] * a[15];
    
    tmp[12] = b[12] * a[0] + b[13] * a[4] + b[14] * a[8] + b[15] * a[12];
    tmp[13] = b[12] * a[1] + b[13] * a[5] + b[14] * a[9] + b[15] * a[13];
    tmp[14] = b[12] * a[2] + b[13] * a[6] + b[14] * a[10] + b[15] * a[14];
    tmp[15] = b[12] * a[3] + b[13] * a[7] + b[14] * a[11] + b[15] * a[15];

    *out = tmp;
}

#[allow(dead_code)]
pub fn inverse_matrix(m: &[f32; 16], out: &mut [f32; 16]) -> bool {
    adjoint_matrix(&m, out);

    let det = determinant4x4(m);
    if det == 0f32 {
        return false;
    }

    for i in 0..16 {
        out[i] = out[i] / det;
    }
    true
}

#[allow(dead_code)]
pub fn adjoint_matrix(m: &[f32; 16], out: &mut [f32; 16]) {
    let mut tmp: [f32; 16] = unsafe { mem::uninitialized() };

    tmp[0]  =   determinant3x3(m[5], m[9], m[13], m[6], m[10], m[14], m[7], m[11], m[15]);
    tmp[4]  = - determinant3x3(m[4], m[8], m[12], m[6], m[10], m[14], m[7], m[11], m[15]);
    tmp[8]  =   determinant3x3(m[4], m[8], m[12], m[5], m[9], m[13], m[7], m[11], m[15]);
    tmp[12] = - determinant3x3(m[4], m[8], m[12], m[5], m[9], m[13], m[6], m[10], m[14]);

    tmp[1]  = - determinant3x3(m[1], m[9], m[13], m[2], m[10], m[14], m[3], m[11], m[15]);
    tmp[5]  =   determinant3x3(m[0], m[8], m[12], m[2], m[10], m[14], m[3], m[11], m[15]);
    tmp[9]  = - determinant3x3(m[0], m[8], m[12], m[1], m[9], m[13], m[3], m[11], m[15]);
    tmp[13] =   determinant3x3(m[0], m[8], m[12], m[1], m[9], m[13], m[2], m[10], m[14]);

    tmp[2]  =   determinant3x3(m[1], m[5], m[13], m[2], m[6], m[14], m[3], m[7], m[15]);
    tmp[6]  = - determinant3x3(m[0], m[4], m[12], m[2], m[6], m[14], m[3], m[7], m[15]);
    tmp[10] =   determinant3x3(m[0], m[4], m[12], m[1], m[5], m[13], m[3], m[7], m[15]);
    tmp[14] = - determinant3x3(m[0], m[4], m[12], m[1], m[5], m[13], m[2], m[6], m[14]);

    tmp[3]  = - determinant3x3(m[1], m[5], m[9], m[2], m[6], m[10], m[3], m[7], m[11]);
    tmp[7]  =   determinant3x3(m[0], m[4], m[8], m[2], m[6], m[10], m[3], m[7], m[11]);
    tmp[11] = - determinant3x3(m[0], m[4], m[8], m[1], m[5], m[9], m[3], m[7], m[11]);
    tmp[15] =   determinant3x3(m[0], m[4], m[8], m[1], m[5], m[9], m[2], m[6], m[10]);
    
    *out = tmp;
}

#[allow(dead_code)]
pub fn determinant4x4(m: &[f32; 16]) -> f32 {
    m[0] * determinant3x3(m[5], m[9], m[13], m[6], m[10], m[14], m[7], m[11], m[15])
    - m[1] * determinant3x3(m[4], m[8], m[12], m[6], m[10], m[14], m[7], m[11], m[15])
    + m[2] * determinant3x3(m[4], m[8], m[12], m[5], m[9], m[13], m[7], m[11], m[15])
    - m[3] * determinant3x3(m[4], m[8], m[12], m[5], m[9], m[13], m[6], m[10], m[14])
}

#[allow(dead_code)]
fn determinant3x3(a1: f32, a2: f32, a3: f32, b1: f32, b2: f32, b3: f32, c1: f32, c2: f32, c3: f32) -> f32 {
    a1 * determinant2x2(b2, b3, c2, c3)
    - b1 * determinant2x2(a2, a3, c2, c3)
    + c1 * determinant2x2(a2, a3, b2, b3)
}

#[allow(dead_code)]
#[inline]
fn determinant2x2(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * d - b * c
}

// Adapted from http://www.euclideanspace.com/maths/geometry/rotations/conversions/matrixToQuaternion/index.htm
#[allow(dead_code)]
#[inline]
pub fn matrix_to_quat(matrix: &[f32; 16]) -> [f32; 4] {
    let m: &[[f32; 4]; 4] = unsafe { mem::transmute(matrix) };
    let w = f32::max(0.0, 1.0 + m[0][0] + m[1][1] + m[2][2]).sqrt() * 0.5;
    let mut x = f32::max(0.0, 1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 0.5;
    let mut y = f32::max(0.0, 1.0 - m[0][0] + m[1][1] - m[2][2]).sqrt() * 0.5;
    let mut z = f32::max(0.0, 1.0 - m[0][0] - m[1][1] + m[2][2]).sqrt() * 0.5;

    x = copysign(x, m[2][1] - m[1][2]);
    y = copysign(y, m[0][2] - m[2][0]);
    z = copysign(z, m[1][0] - m[0][1]);

    [x, y, z, w]
}

#[allow(dead_code)]
#[inline]
pub fn copysign(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        0.0
    } else {
        a.abs() * b.signum()
    }
}
