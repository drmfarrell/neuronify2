use hecs::World;

pub enum Contents {
    Saved {
        file: &'static str,
        bytes: &'static [u8],
    },
}

macro_rules! saved {
    ($file:literal) => {
        Contents::Saved {
            file: $file,
            bytes: include_bytes!(concat!("../examples/", $file)),
        }
    };
}

pub struct Example {
    pub name: &'static str,
    pub description: &'static str,
    pub contents: Contents,
}

impl Example {
    pub fn file(&self) -> &'static str {
        match self.contents {
            Contents::Saved { file, .. } => file,
        }
    }

    /// The tool a circuit opens in: Stimulate, so a first press fires a neuron,
    /// and Edit for the logic challenge, where the first job is drawing connections.
    pub fn start_tool(&self) -> crate::tools::Tool {
        if self.file().contains("8-logic-challenge") {
            crate::tools::Tool::Edit
        } else {
            crate::tools::Tool::Stimulate
        }
    }

    pub fn spawn(&self) -> Result<World, String> {
        match self.contents {
            Contents::Saved { bytes, .. } => Ok(crate::serialization::load(bytes)?.world),
        }
    }
}

pub fn find(file: &str) -> Option<&'static Example> {
    EXAMPLE_CATEGORIES
        .iter()
        .flat_map(|category| category.examples)
        .chain(TUTORIAL_PAGES)
        .find(|example| example.file() == file)
}

pub struct ExampleCategory {
    pub name: &'static str,
    pub examples: &'static [Example],
}

/// Pages reached only by a Next link or by address, never cards on the start page: the short pages
/// of "0. Using Neuronify" and the upstream tutorials 4 to 11, which this fork does not offer.
pub const TUTORIAL_PAGES: &[Example] = &[
    Example {
        name: "0. Using Neuronify: reading a voltage",
        description: "",
        contents: saved!("tutorial/0b_reading_a_voltage.neuronify"),
    },
    Example {
        name: "0. Using Neuronify: changing a value",
        description: "",
        contents: saved!("tutorial/0c_changing_a_value.neuronify"),
    },
    Example {
        name: "0. Using Neuronify: drawing a connection",
        description: "",
        contents: saved!("tutorial/0d_drawing_a_connection.neuronify"),
    },
    Example {
        name: "0. Using Neuronify: the playback bar",
        description: "",
        contents: saved!("tutorial/0e_playback_bar.neuronify"),
    },
    Example {
        name: "Choosing sides",
        description: "",
        contents: saved!("tutorial/4_choosing.neuronify"),
    },
    Example {
        name: "Sharpening edges",
        description: "",
        contents: saved!("tutorial/5_sharpening_edges.neuronify"),
    },
    Example {
        name: "Keeping a rhythm",
        description: "",
        contents: saved!("tutorial/6_keeping_a_rhythm.neuronify"),
    },
    Example {
        name: "Axons",
        description: "",
        contents: saved!("tutorial/7_axons.neuronify"),
    },
    Example {
        name: "Myelin",
        description: "",
        contents: saved!("tutorial/8_myelin.neuronify"),
    },
    Example {
        name: "Dendrites",
        description: "",
        contents: saved!("tutorial/9_dendrites.neuronify"),
    },
    Example {
        name: "Coincidence",
        description: "",
        contents: saved!("tutorial/10_coincidence.neuronify"),
    },
    Example {
        name: "Inhibition on a branch",
        description: "",
        contents: saved!("tutorial/11_inhibition_on_a_branch.neuronify"),
    },
];

pub const EXAMPLE_CATEGORIES: &[ExampleCategory] = &[
    ExampleCategory {
        name: "Start here: learn Neuronify",
        examples: &[
            Example {
                name: "0. Using Neuronify",
                description: "Press a sensor, read a voltage, change a value, draw a connection and use the playback bar.",
                contents: saved!("tutorial/0_using_neuronify.neuronify"),
            },
            Example {
                name: "1. Spikes",
                description: "Press the touch sensor to make a neuron fire.",
                contents: saved!("tutorial/1_spikes.neuronify"),
            },
            Example {
                name: "2. Leak currents",
                description: "One press is not enough to make a neuron fire, because charge leaks away.",
                contents: saved!("tutorial/2_leaks.neuronify"),
            },
            Example {
                name: "3. Push and pull",
                description: "Excitatory neurons push others toward firing. Inhibitory neurons pull them back.",
                contents: saved!("tutorial/3_push_and_pull.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "A&P Lab 10: synaptic summation",
        examples: &[
            Example {
                name: "1. Threshold",
                description: "Raise the current into one neuron until it fires.",
                contents: saved!("lab10/1-threshold.neuronify"),
            },
            Example {
                name: "2. One input, two presses",
                description: "One press fades away. Two quick presses add up and fire the output.",
                contents: saved!("lab10/2-one-input.neuronify"),
            },
            Example {
                name: "3. Steady rate",
                description: "Change how fast the input fires and find the rate where the output fires.",
                contents: saved!("lab10/3-steady-rate.neuronify"),
            },
            Example {
                name: "4. Three inputs at once",
                description: "One input is too weak and two are too weak. All three together fire the output.",
                contents: saved!("lab10/4-spatial.neuronify"),
            },
            Example {
                name: "5. Synaptic strength",
                description: "Make one connection strong enough that its input alone fires the output.",
                contents: saved!("lab10/5-synaptic-strength.neuronify"),
            },
            Example {
                name: "6. Inhibition",
                description: "An inhibitory neuron pulls the output down and cancels some excitatory inputs.",
                contents: saved!("lab10/6-inhibition.neuronify"),
            },
            Example {
                name: "7. Refractory period (optional)",
                description: "The output cannot fire again right after a spike.",
                contents: saved!("lab10/7-refractory-optional.neuronify"),
            },
            Example {
                name: "Take-home: logic challenge",
                description: "Connect the inputs to the output so that it behaves like AND, then OR.",
                contents: saved!("lab10/8-logic-challenge.neuronify"),
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::LeakyNeuron;

    /// Upstream examples kept in the folder but not offered to students in this fork.
    const NOT_LISTED: &[&str] = &[
        "adaptation.neuronify",
        "adaptive_inhibition.neuronify",
        "bursting.neuronify",
        "direction_selectivity.neuronify",
        "disinhibition.neuronify",
        "feedback_inhibition.neuronify",
        "feedback_loops.neuronify",
        "firing_threshold.neuronify",
        "half_center_oscillator.neuronify",
        "lateral_inhibition.neuronify",
        "listening_to_a_neuron.neuronify",
        "receptive_fields.neuronify",
        "refractory_period.neuronify",
        "reverberation.neuronify",
        "summation.neuronify",
    ];

    #[test]
    fn the_start_page_has_only_the_tutorial_and_the_lab() {
        let names: Vec<&str> = EXAMPLE_CATEGORIES.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            [
                "Start here: learn Neuronify",
                "A&P Lab 10: synaptic summation"
            ]
        );
        let lab = &EXAMPLE_CATEGORIES[1];
        assert_eq!(lab.examples.len(), 8);
        for example in lab.examples {
            let expected = if example.file().contains("8-logic") {
                crate::tools::Tool::Edit
            } else {
                crate::tools::Tool::Stimulate
            };
            assert_eq!(example.start_tool(), expected);
        }
    }

    #[test]
    fn examples_are_found_by_their_file() {
        assert_eq!(
            find("lab10/2-one-input.neuronify").unwrap().name,
            "2. One input, two presses"
        );
        assert!(find("missing.nfy").is_none());
    }

    #[test]
    fn every_bundled_example_file_is_listed() {
        let listed: Vec<&str> = EXAMPLE_CATEGORIES
            .iter()
            .flat_map(|category| category.examples)
            .chain(TUTORIAL_PAGES)
            .map(Example::file)
            .collect();
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
        for entry in std::fs::read_dir(folder).unwrap() {
            let file = entry.unwrap().file_name().to_string_lossy().into_owned();
            if file.ends_with(".neuronify") && !NOT_LISTED.contains(&file.as_str()) {
                assert!(listed.contains(&file.as_str()), "{file} is not listed");
            }
        }
    }

    #[test]
    fn every_example_has_neurons() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            let neurons = world.query::<&LeakyNeuron>().iter().count();
            assert!(neurons > 0, "{} has no neurons", example.name);
        }
    }
}
