
extern crate phonon_sys;
extern crate wav;
//extern crate libc;

use phonon_sys::*;
use std::os::raw::*;
use std::ffi::CString;
use wav::{BitDepth};

/*
const SAMPLING_RATE: c_int = 44100;
const FRAME_SIZE: c_int = 1024;
*/

fn vf_to_u8(v: &Vec<f32>) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

fn u8_to_vf(v: &Vec<u8>) -> &[f32] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const f32, v.len() / 4) }
}

fn main() {
    let data = std::fs::read("assets/mezame.raw").unwrap();
    println!("data: {:?}", &data[1_000_000..1_001_024]);
    let vf_data = u8_to_vf(&data);
    println!("data: {:?}", &vf_data[1_000_000..1_001_024]);

    let mut input_audio = Vec::new();
    input_audio.extend_from_slice(vf_data);

    unsafe {

        let mut context: IPLhandle = std::ptr::null_mut();
        let err = iplCreateContext(None, None, None, &mut context as *mut _);
        println!("create context err: {:?}", err);

        let sampling_rate = 44100;
        let frame_size = 1024;

        let render_settings = IPLRenderingSettings {
            samplingRate: sampling_rate,
            frameSize: frame_size,
            convolutionType: IPLConvolutionType_IPL_CONVOLUTIONTYPE_PHONON,
        };

        let mut renderer: IPLhandle = std::ptr::null_mut();
        let hrtf_params = IPLHrtfParams {
            type_: IPLHrtfDatabaseType_IPL_HRTFDATABASETYPE_DEFAULT,
            hrtfData: std::ptr::null_mut(),
            sofaFileName: CString::default().into_raw(),
        };

        let err = iplCreateBinauralRenderer(context, render_settings, hrtf_params, &mut renderer as *mut _);
        println!("create renderer err: {:?}", err);

        let mono = IPLAudioFormat {
            channelLayoutType: IPLChannelLayoutType_IPL_CHANNELLAYOUTTYPE_SPEAKERS,
            channelLayout: IPLChannelLayout_IPL_CHANNELLAYOUT_MONO,
            channelOrder: IPLChannelOrder_IPL_CHANNELORDER_INTERLEAVED,
            ambisonicsNormalization: IPLAmbisonicsNormalization_IPL_AMBISONICSNORMALIZATION_FURSEMALHAM,
            ambisonicsOrder: 0,
            ambisonicsOrdering: IPLAmbisonicsOrdering_IPL_AMBISONICSORDERING_FURSEMALHAM,
            numSpeakers: 1,
            speakerDirections: std::ptr::null_mut(),
        };

        let stereo = IPLAudioFormat {
            channelLayoutType: IPLChannelLayoutType_IPL_CHANNELLAYOUTTYPE_SPEAKERS,
            channelLayout: IPLChannelLayout_IPL_CHANNELLAYOUT_STEREO,
            channelOrder: IPLChannelOrder_IPL_CHANNELORDER_INTERLEAVED,
            ambisonicsNormalization: IPLAmbisonicsNormalization_IPL_AMBISONICSNORMALIZATION_FURSEMALHAM,
            ambisonicsOrder: 0,
            ambisonicsOrdering: IPLAmbisonicsOrdering_IPL_AMBISONICSORDERING_FURSEMALHAM,
            numSpeakers: 2,
            speakerDirections: std::ptr::null_mut(),
        };

        /*
            IPLAudioFormat mono;
            mono.channelLayoutType  = IPL_CHANNELLAYOUTTYPE_SPEAKERS;
            mono.channelLayout      = IPL_CHANNELLAYOUT_MONO;
            mono.channelOrder       = IPL_CHANNELORDER_INTERLEAVED;
            IPLAudioFormat stereo;
            stereo.channelLayoutType  = IPL_CHANNELLAYOUTTYPE_SPEAKERS;
            stereo.channelLayout      = IPL_CHANNELLAYOUT_STEREO;
            stereo.channelOrder       = IPL_CHANNELORDER_INTERLEAVED;
        */

        let mut effect: IPLhandle = std::ptr::null_mut();
        iplCreateBinauralEffect(renderer, mono, stereo, &mut effect as *mut _);


        /*
            IPLhandle effect{ nullptr };
            iplCreateBinauralEffect(renderer, mono, stereo, &effect);
        */

        let mut in_buffer = IPLAudioBuffer {
            format: mono,
            numSamples: frame_size,
            interleavedBuffer: input_audio[100_000..].as_mut_ptr(),
            deinterleavedBuffer: std::ptr::null_mut(),
        };

        let mut output_audio_frame = vec![0.0; (2 * frame_size) as usize];

        let out_buffer = IPLAudioBuffer {
            format: stereo,
            numSamples: frame_size,
            interleavedBuffer: output_audio_frame.as_mut_ptr(),
            deinterleavedBuffer: std::ptr::null_mut(),
        };

        let mut output_audio: Vec<f32> = Vec::new();

        let num_frames = input_audio.len() / frame_size as usize;


        for i in 0..num_frames {
            let x = f32::sin(std::f32::consts::PI / 180.0f32 * i as f32) * 3.0;
            let z = f32::cos(std::f32::consts::PI / 180.0f32 * i as f32) * 3.0;
            iplApplyBinauralEffect(effect, renderer, in_buffer, IPLVector3 { x: x, y: 0.0, z: z }, IPLHrtfInterpolation_IPL_HRTFINTERPOLATION_NEAREST, 1.0, out_buffer);
            for piece in &output_audio_frame {
                output_audio.push(*piece);
            }
            in_buffer.interleavedBuffer = input_audio[i * frame_size as usize..].as_mut_ptr();
        }

        println!("{:?}", &output_audio[1_000_000..1_001_024]);
        let byte_data = vf_to_u8(&output_audio);
        std::fs::write("assets/output_mezame.raw", byte_data).unwrap();

        iplDestroyBinauralEffect(&mut effect);
        iplDestroyBinauralRenderer(&mut renderer);
        iplDestroyContext(&mut context);
        iplCleanup();
    }
}