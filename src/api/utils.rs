use std::mem;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
use std::sync::atomic::Ordering::SeqCst;

static DEVICE_ID_COUNTER: AtomicUsize  = ATOMIC_USIZE_INIT;

// Generates a unique identifier for any VRDevice
pub fn new_device_id() -> u64 {
    DEVICE_ID_COUNTER.fetch_add(1, SeqCst) as u64
}

// Multiply 4x4 matrices
pub fn multiply_matrix(a: &[f32; 16], b: &[f32; 16], out: &mut [f32;16]) {
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
