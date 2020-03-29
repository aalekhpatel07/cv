use crate::derivatives;
use crate::evolution::{Config, EvolutionStep};
use crate::image::{GrayFloatImage, ImageFunctions};
use itertools::izip;

fn compute_multiscale_derivatives_for_evolution(evolution: &mut EvolutionStep, sigma_size: u32) {
    evolution.Lx = derivatives::scharr_horizontal(&evolution.Lsmooth, sigma_size);
    evolution.Ly = derivatives::scharr_vertical(&evolution.Lsmooth, sigma_size);
    evolution.Lxx = derivatives::scharr_horizontal(&evolution.Lx, sigma_size);
    evolution.Lyy = derivatives::scharr_vertical(&evolution.Ly, sigma_size);
    evolution.Lxy = derivatives::scharr_vertical(&evolution.Lx, sigma_size);
}

fn compute_multiscale_derivatives(evolutions: &mut Vec<EvolutionStep>, options: Config) {
    for evolution in evolutions.iter_mut() {
        // The image decreases in size by a factor which is 2^octave.
        let ratio = 2.0f64.powi(evolution.octave as i32);
        // The scale of the edge filter.
        let sigma_size = f64::round(evolution.esigma * options.derivative_factor / ratio) as u32;
        compute_multiscale_derivatives_for_evolution(evolution, sigma_size);
    }
}

/// Compute the detector response - the determinant of the Hessian - and save the result
/// in the evolutions.
///
/// # Arguments
/// * `evolutions` - The computed evolutions.
/// * `options` - The options
#[allow(non_snake_case)]
pub fn detector_response(evolutions: &mut Vec<EvolutionStep>, options: Config) {
    compute_multiscale_derivatives(evolutions, options);
    for evolution in evolutions.iter_mut() {
        let ratio = f64::powi(2.0, evolution.octave as i32);
        let sigma_size = f64::round(evolution.esigma * options.derivative_factor / ratio);
        let sigma_size_quat = sigma_size.powi(4) as f32;
        evolution.Ldet = GrayFloatImage::new(evolution.Lxx.width(), evolution.Lxx.height());
        for (Ldet, Lxx, Lyy, Lxy) in izip!(
            evolution.Ldet.iter_mut(),
            evolution.Lxx.iter(),
            evolution.Lyy.iter(),
            evolution.Lxy.iter(),
        ) {
            *Ldet = ((Lxx * Lyy) - (Lxy * Lxy)) * sigma_size_quat;
        }
    }
}
