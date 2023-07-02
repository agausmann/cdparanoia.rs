use std::{ffi::c_int, io::SeekFrom};

use anyhow::{ensure, Context};
use cdparanoia::{CdromDrive, CdromParanoia, ParanoiaMode, Verbosity};
use hound::{SampleFormat, WavSpec, WavWriter};
use libc::c_long;

fn main() -> anyhow::Result<()> {
    let drive =
        CdromDrive::find_a_cdrom(Verbosity::PrintIt).context("failed to find a CD drive.")?;
    drive.open().context("failed to open drive")?;
    let mut paranoia = CdromParanoia::init(drive);

    paranoia.set_mode(ParanoiaMode::FULL);
    paranoia
        .drive()
        .set_verbosity(Verbosity::PrintIt, Verbosity::PrintIt);

    let track = 1;
    ensure!(paranoia.drive().track_audiop(track)?);

    let first_sector = paranoia.drive().track_first_sector(track)?;
    let last_sector = paranoia.drive().track_last_sector(track)?;
    let num_channels = paranoia.drive().track_channels(track)?;

    let mut output = WavWriter::create(
        "track01.wav",
        WavSpec {
            channels: num_channels.try_into().unwrap(),
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        },
    )
    .context("failed to create track01.wav")?;

    paranoia.seek(SeekFrom::Start(first_sector))?;

    eprintln!(
        "track01: Ripping {} sectors",
        last_sector - first_sector + 1
    );

    for _ in first_sector..=last_sector {
        let data = paranoia.read(event_callback);

        for &sample in data {
            output.write_sample(sample)?;
        }

        if let Some(message) = paranoia.drive().messages() {
            let message_string = message.to_string_lossy();
            for line in message_string.lines() {
                eprintln!("MSG: {}", line);
            }
        }
        if let Some(error) = paranoia.drive().errors() {
            let error_string = error.to_string_lossy();
            for line in error_string.lines() {
                eprintln!("ERR: {}", line);
            }
        }
    }

    Ok(())
}

extern "C" fn event_callback(position: c_long, event: c_int) {
    use cdparanoia_sys::*;
    let description = match event as u32 {
        PARANOIA_CB_READ => "read",
        PARANOIA_CB_VERIFY => "verifying jitter",
        PARANOIA_CB_FIXUP_EDGE => "fixed edge jitter",
        PARANOIA_CB_FIXUP_ATOM => "fixed atom jitter",
        PARANOIA_CB_SCRATCH => "scratch",
        PARANOIA_CB_REPAIR => "repair",
        PARANOIA_CB_SKIP => "skip exhausted retry",
        PARANOIA_CB_DRIFT => "drift exhausted retry",
        PARANOIA_CB_BACKOFF => "backoff",
        PARANOIA_CB_OVERLAP => "dynamic overlap adjust",
        PARANOIA_CB_FIXUP_DROPPED => "fixed dropped bytes",
        PARANOIA_CB_FIXUP_DUPED => "fixed duplicated bytes",
        PARANOIA_CB_READERR => "read error",
        PARANOIA_CB_CACHEERR => "cache error",
        _ => "unknown",
    };
    eprintln!("EV: position {}: {} ({})", position, description, event);
}
