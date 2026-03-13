use std::fs::File;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use rusty_chromaprint::{Configuration, Fingerprinter, FingerprintCompressor};
use data_encoding::BASE64URL_NOPAD;

#[test]
fn debug_fingerprint() {
    let path = std::path::Path::new("C:\\Users\\ANDRES\\OneDrive\\Music\\Music\\'guaya' arcangel ft daddy yankee.mp3");
    
    let file = File::open(path).unwrap();
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .unwrap();
    
    let mut format = probed.format;
    let track = format.default_track().unwrap();
    let track_id = track.id;
    
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .unwrap();
    
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
    
    let mut printer = Fingerprinter::new(&Configuration::preset_test1());
    printer.start(sample_rate, channels as u32).unwrap();
    
    let mut sample_buf = None;
    let mut decoded_seconds = 0.0f64;
    
    while decoded_seconds < 120.0 {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };
        if packet.track_id() != track_id { continue; }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                }
                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(decoded);
                    let samples = buf.samples();
                    printer.consume(samples);
                    decoded_seconds += (samples.len() as f64 / channels as f64) / sample_rate as f64;
                }
            }
            Err(_) => continue,
        }
    }
    
    printer.finish();
    let fp = printer.fingerprint();
    println!("Fingerprint u32 count: {}", fp.len());
    println!("Duration: {:.2}s", decoded_seconds);
    
    // Method 1: FingerprintCompressor + BASE64URL_NOPAD
    let config = Configuration::preset_test1();
    let compressor = FingerprintCompressor::from(&config);
    let compressed = compressor.compress(fp);
    let b64_compressed = BASE64URL_NOPAD.encode(&compressed);
    println!("BASE64URL (compressed, {} bytes): {}...", compressed.len(), &b64_compressed[..20.min(b64_compressed.len())]);
    
    // Method 2: raw u32 LE bytes + BASE64URL_NOPAD
    let raw_bytes: Vec<u8> = fp.iter().flat_map(|&v| v.to_le_bytes()).collect();
    let b64_raw = BASE64URL_NOPAD.encode(&raw_bytes);
    println!("BASE64URL (raw, {} bytes): {}...", raw_bytes.len(), &b64_raw[..20.min(b64_raw.len())]);
    
    // Check what the real acoustid fingerprint looks like (starts with AQAB)
    println!("Real AcoustID example starts with: AQABz0qU...");
    println!("Our compressed starts with: {}", &b64_compressed[..10.min(b64_compressed.len())]);
    println!("Our raw starts with: {}", &b64_raw[..10.min(b64_raw.len())]);
}
