pub mod sox;
use crate::wave::Stereo;

/// Copy the stereo `f64` samples to bytes, interleaving the left and right samples.
///
/// Could probably be implemented with some sort of unsafe transmute,
/// but copying is safe and likely not the bottleneck.
///
/// Returns the number of samples that were actually copied.
/// Might be less than the number of input samples if the output buffer was not large enough.
pub fn copy_f64_bytes(audio: &[Stereo<f64>], bytes: &mut [u8]) -> usize {
    let mut processed = 0;
    for (sample, target) in audio.iter().zip(bytes.chunks_exact_mut(16)) {
        target[0..8].copy_from_slice(&sample.left.to_le_bytes());
        target[8..16].copy_from_slice(&sample.left.to_le_bytes());
        processed += 1;
    }
    processed
}
