use std::{
    collections::{BTreeSet, HashMap},
    ops::Mul,
    time::Duration,
};

use legato_core::engine::{builder::AddNode, node::FrameSize};
use typenum::{Prod, U2};

use crate::ast::{Object, Value};

/// ValidationError covers logical issues
/// when lowering from the AST to the IR.
///
/// Typically, these might be bad parameters,
/// bad values, nodes that don't exist, etc.
#[derive(Clone, PartialEq, Debug)]
pub enum ValidationError {
    NodeNotFound(String),
    InvalidParameter(String),
    MissingRequiredParameters(String),
    MissingRequiredParameter(String),
}

/// Convenience struct to help lower from Objects
/// to parameters that can create nodes and pipes.
pub struct Params<'a>(pub &'a Object);

impl<'a> Params<'a> {
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match self.0.get(key) {
            Some(Value::F32(x)) => Some(*x),
            Some(Value::I32(x)) => Some(*x as f32),
            Some(Value::U32(x)) => Some(*x as f32),
            Some(x) => panic!("Expected F32 param, found {:?}", x),
            _ => None,
        }
    }

    // Just ms for the time being
    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        match self.0.get(key) {
            Some(Value::F32(ms)) => Some(Duration::from_secs_f32(ms / 1000.0)),
            Some(Value::I32(ms)) => Some(Duration::from_millis(*ms as u64)),
            Some(Value::U32(ms)) => Some(Duration::from_millis(*ms as u64)),
            Some(x) => panic!("Expected F32 or I32 param for ms, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        match self.0.get(key) {
            Some(Value::U32(s)) => Some(*s),
            Some(x) => panic!("Expected U32 param, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        match self.0.get(key) {
            Some(Value::Str(s)) => Some(s.clone()),
            Some(Value::Ident(i)) => Some(i.clone()),
            Some(x) => panic!("Expected str param, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(Value::Bool(b)) => Some(*b),
            Some(x) => panic!("Expected bool param, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_object(&self, key: &str) -> Option<Object> {
        match self.0.get(key) {
            Some(Value::Obj(o)) => Some(o.clone()),
            Some(x) => panic!("Expected object param, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_array(&self, key: &str) -> Option<Vec<Value>> {
        match self.0.get(key) {
            Some(Value::Array(v)) => Some(v.clone()),
            Some(x) => panic!("Expected array param, found {:?}", x),
            _ => None,
        }
    }

    pub fn get_array_f32(&self, key: &str) -> Option<Vec<f32>> {
        let arr = match self.0.get(key) {
            Some(Value::Array(v)) => Some(v.clone()),
            Some(x) => panic!("Expected array param, found {:?}", x),
            _ => None,
        };

        Some(
            arr.unwrap()
                .into_iter()
                .map(|x| match x {
                    Value::F32(x) => x,
                    Value::I32(x) => x as f32,
                    Value::U32(x) => x as f32,
                    _ => panic!("Unexpected value in f32 array {:?}", x),
                })
                .collect(),
        )
    }

    pub fn get_array_duration_ms(&self, key: &str) -> Option<Vec<Duration>> {
        let arr = match self.0.get(key) {
            Some(Value::Array(v)) => Some(v.clone()),
            Some(x) => panic!("Expected array param, found {:?}", x),
            _ => None,
        };

        Some(
            arr.unwrap()
                .into_iter()
                .map(|x| match x {
                    Value::F32(x) => Duration::from_secs_f32(x / 1000.0),
                    Value::I32(x) => Duration::from_millis(x as u64),
                    Value::U32(x) => Duration::from_millis(x as u64),
                    _ => panic!("Unexpected value in f32 array {:?}", x),
                })
                .collect(),
        )
    }

    pub fn validate(&self, allowed: &BTreeSet<String>) -> Result<(), ValidationError> {
        // Iterate through keys. If we have one that's not allowed, return an error
        for k in self.0.keys() {
            if !allowed.contains(k) {
                return Err(ValidationError::InvalidParameter(format!(
                    "Could not find parameter with name {}",
                    k
                )));
            }
        }
        Ok(())
    }

    pub fn required(&self, required: &BTreeSet<String>) -> Result<(), ValidationError> {
        for k in required {
            if !self.0.contains_key(k) {
                return Err(ValidationError::MissingRequiredParameter(format!(
                    "Missing required perameter {}",
                    k,
                )));
            }
        }
        Ok(())
    }
}

macro_rules! param_list {
    ($($param:expr),* $(,)?) => {
        {
            let mut set = BTreeSet::new();
            $(set.insert(String::from($param));)*
            set
        }
    };
}

/// A node registry trait that let's users extend the graph logic
/// to make their own node namespaces. For example, you could make a
/// reverb namespace that has a bunch of primitives you might need,
/// or you could make a physics namespace with physics logic.
trait NodeRegistry<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    fn lower_to_ir(
        &self,
        name: String,
        params: Option<&Params>,
    ) -> Result<AddNode<AF, CF>, ValidationError>;
}

/// The default container of node registries.
///
/// Users can make their own registries with their
/// own pairs. This means that when you're using
/// the graph, you can choose which namespaces
/// and nodes are in there, extend it on your own, etc.
///
/// To do this yourself, implement the NodeRegistry trait.
/// You can then extend the Legato registry container,
/// or make your own at a later time.
struct LegatoRegistryContainer<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    namespaces: HashMap<String, Box<dyn NodeRegistry<AF, CF>>>,
}

impl<AF, CF> LegatoRegistryContainer<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    pub fn new() -> Self {
        let mut namespaces = HashMap::new();
        namespaces.insert(
            String::from("audio"),
            Box::new(AudioRegistry::default()) as Box<dyn NodeRegistry<AF, CF>>,
        );
        Self { namespaces }
    }
}

/// One of the default registries, audio deals
/// with common audio effects. This may be renamed
/// in the future.
#[derive(Default)]
struct AudioRegistry;

impl<AF, CF> NodeRegistry<AF, CF> for AudioRegistry
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    fn lower_to_ir(
        &self,
        name: String,
        params: Option<&Params>,
    ) -> Result<AddNode<AF, CF>, ValidationError> {
        // TODO: Not in love with this. Maybe a macro or reflection library?
        match name.as_str() {
            // Osc
            "sine_mono" => {
                if let Some(p) = params {
                    p.validate(&param_list!("freq")).unwrap();
                }
                let freq = params.and_then(|p| p.get_f32("freq")).unwrap_or(440.0);
                Ok(AddNode::SineMono { freq })
            }
            "sine_stereo" => {
                if let Some(p) = params {
                    p.validate(&param_list!("freq")).unwrap();
                }
                let freq = params.and_then(|p| p.get_f32("freq")).unwrap_or(440.0);
                Ok(AddNode::SineStereo { freq })
            }
            // Fan mono to stereo
            "stereo" => Ok(AddNode::Stereo),
            "sampler_mono" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Sampler requires sample key",
                    )))
                    .unwrap();

                let p_list = param_list!("sample_name");

                p.validate(&p_list).unwrap();
                p.required(&p_list).unwrap();

                let sampler_name = p.get_str("sampler_name").unwrap();

                Ok(AddNode::SamplerMono {
                    sampler_name: sampler_name,
                })
            }
            "sampler_stereo" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Sampler requires sample key",
                    )))
                    .unwrap();

                let p_list = param_list!("sample_name");

                p.validate(&p_list).unwrap();
                p.required(&p_list).unwrap();

                let sampler_name = p.get_str("sampler_name").unwrap();

                Ok(AddNode::SamplerStereo {
                    sampler_name: sampler_name,
                })
            }
            // Delays
            "delay_write_mono" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Delay write mono requires delay key",
                    )))
                    .unwrap();

                let p_list = param_list!("delay_name", "delay_length");

                p.validate(&p_list).unwrap();
                p.required(&param_list!("delay_name")).unwrap();

                let delay_name = p.get_str("delay_name").unwrap();
                let delay_length = p
                    .get_duration("delay_length")
                    .unwrap_or(Duration::from_secs(1));

                Ok(AddNode::DelayWriteMono {
                    delay_name: delay_name,
                    delay_length: delay_length,
                })
            }
            "delay_write_stereo" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Delay write mono requires delay key",
                    )))
                    .unwrap();

                let p_list = param_list!("delay_name", "delay_length");

                p.validate(&p_list).unwrap();
                p.required(&param_list!("delay_name")).unwrap();

                let delay_name = p.get_str("delay_name").unwrap();
                let delay_length = p
                    .get_duration("delay_length")
                    .unwrap_or(Duration::from_secs(1));

                Ok(AddNode::DelayWriteStereo {
                    delay_name: delay_name,
                    delay_length: delay_length,
                })
            }
            "delay_read_mono" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Delay read mono requires delay key",
                    )))
                    .unwrap();

                let p_list = param_list!("delay_name", "offsets");

                p.validate(&p_list).unwrap();
                p.required(&param_list!("delay_name")).unwrap();

                let delay_name = p.get_str("delay_name").unwrap();
                let offsets = p
                    .get_array_duration_ms("offsets")
                    .unwrap_or(vec![Duration::from_millis(200); 1]);

                Ok(AddNode::DelayReadMono {
                    delay_name: delay_name,
                    offsets: offsets,
                })
            }
            "delay_read_stereo" => {
                let p = params
                    .ok_or(ValidationError::MissingRequiredParameter(String::from(
                        "Delay read mono requires delay key",
                    )))
                    .unwrap();

                let p_list = param_list!("delay_name", "offsets");

                p.validate(&p_list).unwrap();
                p.required(&param_list!("delay_name")).unwrap();

                let delay_name = p.get_str("delay_name").unwrap();
                let offsets = p
                    .get_array_duration_ms("offsets")
                    .unwrap_or(vec![Duration::from_millis(200); 1]);

                Ok(AddNode::DelayReadStereo {
                    delay_name: delay_name,
                    offsets: offsets,
                })
            }
            // FIR filters
            "fir_mono" => {
                let p = params.ok_or(ValidationError::MissingRequiredParameter(
                    "fir_mono requires coeffs".into(),
                ))?;
                let allowed = param_list!("coeffs");
                p.validate(&allowed)?;
                p.required(&allowed)?;

                let coeffs = p.get_array_f32("coeffs").unwrap();
                Ok(AddNode::FirMono { coeffs })
            }

            "fir_stereo" => {
                let p = params.ok_or(ValidationError::MissingRequiredParameter(
                    "fir_stereo requires coeffs".into(),
                ))?;
                let allowed = param_list!("coeffs");
                p.validate(&allowed)?;
                p.required(&allowed)?;

                let coeffs = p.get_array_f32("coeffs").unwrap();
                Ok(AddNode::FirStereo { coeffs })
            }
            // Ops
            "add_mono" => {
                if let Some(p) = params {
                    p.validate(&param_list!("props"))?;
                }
                let props = params.and_then(|p| p.get_f32("props")).unwrap_or(1.0);
                Ok(AddNode::AddMono { props })
            }

            "add_stereo" => {
                if let Some(p) = params {
                    p.validate(&param_list!("props"))?;
                }
                let props = params.and_then(|p| p.get_f32("props")).unwrap_or(1.0);
                Ok(AddNode::AddStereo { props })
            }

            "mult_mono" => {
                if let Some(p) = params {
                    p.validate(&param_list!("props"))?;
                }
                let props = params.and_then(|p| p.get_f32("props")).unwrap_or(1.0);
                Ok(AddNode::MultMono { props })
            }

            "mult_stereo" => {
                if let Some(p) = params {
                    p.validate(&param_list!("props"))?;
                }
                let props = params.and_then(|p| p.get_f32("props")).unwrap_or(1.0);
                Ok(AddNode::MultStereo { props })
            }

            // Mixers – no parameters
            "stereo_mixer" => Ok(AddNode::StereoMixer),
            "stereo_to_mono" => Ok(AddNode::StereoToMono),
            "two_track_stereo_mixer" => Ok(AddNode::TwoTrackStereoMixer),
            "four_track_stereo_mixer" => Ok(AddNode::FourTrackStereoMixer),
            "eight_track_stereo_mixer" => Ok(AddNode::EightTrackStereoMixer),
            "two_track_mono_mixer" => Ok(AddNode::TwoTrackMonoMixer),
            "four_to_mono_mixer" => Ok(AddNode::FourToMonoMixer),

            // Sweep
            "sweep" => {
                let p = params.ok_or(ValidationError::MissingRequiredParameter(
                    "sweep requires range and duration".into(),
                ))?;

                let allowed = param_list!("range", "duration");
                p.validate(&allowed)?;
                p.required(&allowed)?;

                let range_vals =
                    p.get_array_f32("range")
                        .ok_or(ValidationError::InvalidParameter(
                            "range must be [start,end]".into(),
                        ))?;

                if range_vals.len() != 2 {
                    return Err(ValidationError::InvalidParameter(
                        "range must contain exactly two f32 values".into(),
                    ));
                }

                let duration =
                    p.get_duration("duration")
                        .ok_or(ValidationError::InvalidParameter(
                            "duration required".into(),
                        ))?;

                Ok(AddNode::Sweep {
                    range: (range_vals[0], range_vals[1]),
                    duration,
                })
            }
            _ => Err(ValidationError::NodeNotFound(format!(
                "Could not find node with name {}",
                name
            ))),
        }
    }
}
