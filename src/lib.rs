use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{f32::consts::PI, sync::Arc};


mod editor;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct SimpleBitcrush {
    params: Arc<SimpleBitcrushParams>,
    sample_rate: f32,
    dn_buffer: Vec<f32>,
    bc_buffer: Vec<f32>,
}

#[derive(Params)]
struct SimpleBitcrushParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,

    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    
    #[id = "rate"]
    pub rate: IntParam,
    
    #[id = "cutoff-frequency"]
    pub cutoff_frequency: FloatParam,
    
    #[id = "wet"]
    pub wet: FloatParam,
}

impl Default for SimpleBitcrush {
    fn default() -> Self {
        Self {
            params: Arc::new(SimpleBitcrushParams::default()),
            sample_rate: 0.0,
            dn_buffer: Vec::new(),
            bc_buffer: Vec::new(),
        }
    }
}

impl Default for SimpleBitcrushParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            
            editor_state: editor::default_state(),
            rate: IntParam::new(
                "Rate",
                0,
                IntRange::Linear { min: 1, max: 30 }
            )
            .with_unit(" sample rate"),

            cutoff_frequency: FloatParam::new(
                "Cutoff",
                20000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: 0.25,
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            wet: FloatParam::new(
                "Wet",
                1.0,
                FloatRange::Linear{
                    min: 0.0,
                    max: 1.0
                },
            )
            
        }
    }
}

impl Plugin for SimpleBitcrush {
    const NAME: &'static str = "Simple Bitcrush";
    const VENDOR: &'static str = "Toby Loveridge";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "tobias.loveridge@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.sample_rate = _buffer_config.sample_rate;
        let x: u32 = match _audio_io_layout.main_output_channels {
            Some(x) => u32::from(x),
            None => 0,
        };
        dbg!(x);
        dbg!(self.sample_rate);
        self.dn_buffer = vec![0.0;x as usize];
        self.bc_buffer = vec![0.0;x as usize];
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        
        //dbg!(buffer.channels());
        //dbg!(self.dn_buffer.len());
        //let mut previous_sample: Option<ChannelSamples> = None;
        
        
        for (idx, channel_samples) in buffer.iter_samples().enumerate() {
            // Smoothing is optionally built into the parameters themselves
            //let gain = self.params.gain.smoothed.next();
            let rate = self.params.rate.smoothed.next();
            let cutoff = self.params.cutoff_frequency.smoothed.next();
            let wet = self.params.wet.smoothed.next();

            
            let tan = 0.1 * f32::tan(PI * cutoff / self.sample_rate);
            let  a1 = (tan - 1.0) / (tan + 1.0);
            
            for (i, sample) in channel_samples.into_iter().enumerate() {
                let dry_sample = sample.clone();
                if rate > 1 {

                    if (idx as i32) % rate == 0 {
                        
                        self.bc_buffer[i] = *sample;
                    } else {
                        *sample = self.bc_buffer[i];
                    }
                }
    
                    let allpass_filtered_sample = a1 * *sample + self.dn_buffer[i];
                    self.dn_buffer[i] = *sample - a1 * allpass_filtered_sample;
                    //dbg!(allpass_filtered_sample);
    
                    let filter_output = 0.5 * (*sample + 1.0 * allpass_filtered_sample);
                    
                    *sample = filter_output;

                    *sample = (dry_sample * ((0.0-wet)+1.0)) + (*sample * wet);
            }
            
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for SimpleBitcrush {
    const CLAP_ID: &'static str = "com.your-domain.simple-bitcrush";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple bitcrusher.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for SimpleBitcrush {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(SimpleBitcrush);
nih_export_vst3!(SimpleBitcrush);
