use anyhow::{Result, Context};
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
// use log::info;
use gstreamer::prelude::*;

pub fn play_audio(audio_data: Vec<f32>) -> Result<()> {
    let pipeline = gst::parse_launch(
        "appsrc name=src ! audioconvert ! audioresample ! autoaudiosink"
    ).context("Failed to create GStreamer pipeline")?;

    let src = pipeline.downcast_ref::<gst::Bin>().unwrap()
        .by_name("src")
        .context("Source element not found")?
        .downcast::<gst_app::AppSrc>()
        .map_err(|_| anyhow::anyhow!("Source element is not an AppSrc"))?;

    src.set_caps(Some(&gst_audio::AudioInfo::builder(gst_audio::AudioFormat::F32le, 44100, 1).build().context("Failed to build AudioInfo")?.to_caps().context("Failed to convert AudioInfo to caps")?));
    src.set_format(gst::Format::Time);

    pipeline.set_state(gst::State::Playing).context("Failed to set pipeline to Playing state")?;

    // Calculate the duration
    let duration = gst::ClockTime::from_nseconds((audio_data.len() as u64 * 1_000_000_000) / 44100);

    // Create a new Buffer from the audio data
    let byte_data: Vec<u8> = audio_data.into_iter().flat_map(|f| f.to_le_bytes()).collect();
    let mut buffer = gst::Buffer::from_mut_slice(byte_data);

    // Set the duration on the buffer
    {
        let buffer_ref = buffer.get_mut().unwrap();
        buffer_ref.set_duration(duration);
    }

    src.push_buffer(buffer).context("Failed to push buffer to source")?;
    src.end_of_stream().context("Failed to signal end of stream")?;

    let bus = pipeline.bus().context("Failed to get pipeline bus")?;
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(anyhow::anyhow!("Error: {:?}", err));
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null).context("Failed to set pipeline to Null state")?;
    Ok(())
}