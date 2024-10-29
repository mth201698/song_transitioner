use dasp::Frame;
use dasp::frame::Stereo;
use dasp::sample::Sample;
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use rubato::{FftFixedIn, Resampler};
// use aubio_rs::{OnsetMode, Tempo};
// use aubio_rs::vec::FVec;

const FADE_DURATION: usize = SAMPLE_RATE * 5; // 5 seconds fade at 44.1kHz
const SAMPLE_RATE: usize = 44100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the two WAV files
    let mut song1 = WavReader::open("song1.wav")?;
    let mut song2 = WavReader::open("song2.wav")?;

    // Check the number of channels for song1
    let samples1: Vec<Stereo<f32>> = if song1.spec().channels == 2 {
        song1.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                Stereo::from_fn(|_| sample.to_sample::<f32>())
            })
            .collect()
    } else {
        song1.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                let mono_sample = sample.to_sample::<f32>();
                [mono_sample, mono_sample]
            })
            .collect()
    };

    // Check the number of channels for song2
    let samples2: Vec<Stereo<f32>> = if song2.spec().channels == 2 {
        song2.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                Stereo::from_fn(|_| sample.to_sample::<f32>())
            })
            .collect()
    } else {
        song2.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                let mono_sample = sample.to_sample::<f32>();
                [mono_sample, mono_sample]
            })
            .collect()
    };

    // determine length of pre-shifted song 1
    let sample1_start_transition: usize = samples1.len() - FADE_DURATION;
    let sample2_end_transition: usize = FADE_DURATION;

    // Adjust tempo of song2 to match song1 using rubato
    let ratio = detect_tempo_ratio(&samples1, &samples2)?;
    let mut resampler = FftFixedIn::<f32>::new(
        SAMPLE_RATE,  // Number of channels
        SAMPLE_RATE * (ratio as usize),
        samples2.len() as usize,
        1024,
        1
    )?;
    
    // Convert stereo samples to mono for resampling
    let mono_samples2: Vec<f32> = samples2.iter()
        .map(|frame| (frame[0] + frame[1]) / 2.0)
        .collect();

    // Resample and convert back to stereo
    let resampled = resampler.process(&[&mono_samples2], None)?;
    let adjusted_samples2: Vec<Stereo<f32>> = resampled[0]
        .iter()
        .map(|&s| [s, s])
        .collect();

    // Prepare output WAV writer  
    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut output = WavWriter::create("transition.wav", spec)?;
    
    // Write song1 until the transition point
    for sample in &samples1[..sample1_start_transition] {
        output.write_sample(sample[0].to_sample::<i16>())?;
        output.write_sample(sample[1].to_sample::<i16>())?;
    }

    // Crossfade between song1 and adjusted song2
    for i in 0..FADE_DURATION {
        let fade_out = (1.0 - (i as f32 / FADE_DURATION as f32)).cos() * 0.5 + 0.5;
        let fade_in = (i as f32 / FADE_DURATION as f32).cos() * 0.5 + 0.5;

        let sample1 = samples1[sample1_start_transition + i].scale_amp(fade_out);
        let sample2 = adjusted_samples2[i].scale_amp(fade_in);
        let mixed = [sample1[0] + sample2[0], sample1[1] + sample2[1]];

        output.write_sample(mixed[0].to_sample::<i16>())?;
        output.write_sample(mixed[1].to_sample::<i16>())?;
    }

    // Write the remaining part of song2
    for &sample in &adjusted_samples2[sample2_end_transition..] {
        output.write_sample(sample[0].to_sample::<i16>())?;
        output.write_sample(sample[1].to_sample::<i16>())?;
    }

    println!("DJ-style transition created successfully!");
    Ok(())
}

// // Function to find the first peak (basic energy-based detection)
// fn find_first_peak(samples: &[Stereo<f32>]) -> usize {
//     samples
//         .windows(256)
//         .enumerate()
//         .max_by(|(_, a), (_, b)| a.iter().map(|s| s[0] * s[0] + s[1] * s[1]).sum::<f32>()
//             .partial_cmp(&b.iter().map(|s| s[0] * s[0] + s[1] * s[1]).sum::<f32>())
//             .unwrap())
//         .map(|(i, _)| i)
//         .unwrap_or(0)
// }

// Function to detect tempo ratio between two songs
fn detect_tempo_ratio(
    samples1: &[Stereo<f32>],
    samples2: &[Stereo<f32>],
) -> Result<f64, Box<dyn std::error::Error>> {
    let bpm1 = detect_bpm(samples1);
    let bpm2 = detect_bpm(samples2);
    Ok(bpm1 as f64 / bpm2 as f64)
}

// BPM detection using the `aubio_rs` crate
fn detect_bpm(_samples: &[Stereo<f32>]) -> usize {
    // let mut tempo = Tempo::new(OnsetMode::Complex, 1024, 1, 1).unwrap();
    // let mut total_beats = 0;
    // let mut total_time = 0.0;

    // for frame in samples {
    //     let mono_sample = (frame[0] + frame[1]) / 2.0; // Convert to mono
    //     if tempo.do_result::<FVec>(FVec::from(vec![mono_sample])).is_ok() {
    //         total_beats += 1;
    //     }
    //     total_time += 1.0 / SAMPLE_RATE as f32;
    // }

    // if total_time > 0.0 {
    //     (total_beats as f32 / total_time * 60.0) as usize
    // } else {
    //     0 // Return 0 if no beats are detected
    // }
    120
}
