use dasp::Frame;
use dasp::frame::Stereo;
use dasp::sample::Sample;
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use rubato::{FftFixedIn, Resampler};
use aubio_rs::{OnsetMode, Tempo};
use aubio_rs::vec::FVec;

const FADE_DURATION: usize = SAMPLE_RATE * 15; // 5 seconds fade at 44.1kHz
const SAMPLE_RATE: usize = 44100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the two WAV files
    let mut song1 = WavReader::open("song1.wav")?;
    let mut song2 = WavReader::open("song2.wav")?;

    // Check the number of channels for song1
    let samples1: Vec<Stereo<i16>> = if song1.spec().channels == 2 {
        song1.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                Stereo::from_fn(|_| sample)
            })
            .collect()
    } else {
        song1.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                let mono_sample = sample;
                [mono_sample, mono_sample]
            })
            .collect()
    };

    // Check the number of channels for song2
    let samples2: Vec<Stereo<i16>> = if song2.spec().channels == 2 {
        song2.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                Stereo::from_fn(|_| sample)
            })
            .collect()
    } else {
        song2.samples::<i16>()
            .map(|s| {
                let sample = s.unwrap();
                let mono_sample = sample;
                [mono_sample, mono_sample]
            })
            .collect()
    };

    // determine length of pre-shifted song 1
    let sample1_start_transition: usize = samples1.len() - FADE_DURATION;
    let sample2_end_transition: usize = FADE_DURATION;

    // Calculate initial and final tempo ratios
    let initial_ratio = detect_tempo_ratio(
        &samples1.iter().map(|s| [s[0].to_sample::<f32>(), s[1].to_sample::<f32>()]).collect::<Vec<[f32; 2]>>(),
        &samples2.iter().map(|s| [s[0].to_sample::<f32>(), s[1].to_sample::<f32>()]).collect::<Vec<[f32; 2]>>()
    )?;
    let final_ratio = 1.0; // Target ratio to match song1's tempo

    // Convert stereo samples to mono for resampling
    let mono_samples2: Vec<f32> = samples2.iter()
        .map(|frame| (frame[0].to_sample::<f32>() + frame[1].to_sample::<f32>()) / 2.0)
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
        output.write_sample(sample[0])?;
    }

    // Crossfade with gradual tempo transition
    for i in 0..FADE_DURATION {
        let fade_out = 1.0 - (i as f32 / FADE_DURATION as f32);
        let fade_in = i as f32 / FADE_DURATION as f32;

        // Calculate the current tempo ratio
        let current_ratio = initial_ratio + (final_ratio - initial_ratio) * (i as f64 / FADE_DURATION as f64);

        // Ensure we have enough samples to process
        // Resample the current frame of song2
        let mut resampler = FftFixedIn::<f32>::new(
            SAMPLE_RATE,
            (SAMPLE_RATE as f64 * current_ratio) as usize,
            1,
            1,
            1
        )?;
        let resampled = resampler.process(&[&mono_samples2[i..i+1]], None)?;
        if !resampled[0].is_empty() {
            let sample2 = [resampled[0][0].to_sample::<i16>().scale_amp(fade_in), resampled[0][0].to_sample::<i16>().scale_amp(fade_in)];
    
            let sample1 = samples1[sample1_start_transition + i].scale_amp(fade_out);
            let mixed = [sample1[0] + sample2[0], sample1[1] + sample2[1]];
    
            output.write_sample(mixed[0])?;
        }
    }

    // Write the remaining part of song2
    for &sample in &samples2[sample2_end_transition..] {
        output.write_sample(sample[0])?;
    }

    output.finalize()?;

    println!("DJ-style transition with tempo change created successfully!");
    Ok(())
}

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
fn detect_bpm(samples: &[Stereo<f32>]) -> usize {
    let mut tempo = Tempo::new(OnsetMode::Complex, 2048, 1024, SAMPLE_RATE as u32).unwrap();
    let mut total_beats = 0;
    let mut total_time = 0.0;

    let mut buffer = Vec::new();
    for frame in samples {
        let mono_sample = (frame[0] + frame[1]) / 2.0; // Convert to mono
        buffer.push(mono_sample);

        // Process in chunks
        if buffer.len() >= 64 { // Example buffer size
            let fvec = FVec::from(buffer.clone());
            if tempo.do_result::<FVec>(fvec).is_ok() {
                let current_beat = tempo.get_last();
                if current_beat < 100 { // Adjust the threshold
                    total_beats += 1;
                }
            }
            buffer.clear();
        }
        total_time += 1.0 / SAMPLE_RATE as f32;
    }

    if total_time > 0.0 {
        (total_beats as f32 / total_time * 60.0) as usize
    } else {
        0 // Return 0 if no beats are detected
    }
    // 120
}
