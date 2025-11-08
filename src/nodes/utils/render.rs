use std::{path::Path, time::Duration};

use generic_array::ArrayLength;
use hound::{WavSpec, WavWriter};
use typenum::U0;

use crate::engine::runtime::Runtime;

pub fn render<AF, CF, C>(
    mut runtime: Runtime<AF, CF, C, U0>,
    path: &Path,
    sr: u32,
    time: Duration,
) -> Result<(), hound::Error>
where
    AF: ArrayLength,
    CF: ArrayLength,
    C: ArrayLength,
{
    let dur_in_samples = (time.as_secs_f32() * sr as f32) as usize;
    let mut count = 0_usize;

    let spec = WavSpec {
        channels: C::to_u16(),
        sample_rate: sr,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();

    while count < dur_in_samples {
        let block = runtime.next_block(None);

        for n in 0..AF::USIZE {
            for c in 0..C::USIZE {
                writer.write_sample(block[c][n]).unwrap();
            }
        }
        count += AF::USIZE;
    }

    writer.finalize().unwrap();

    Ok(())
}
