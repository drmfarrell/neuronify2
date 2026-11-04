//! Builds and checks the A&P Lab 10 circuits (examples/lab10/*.neuronify).
//! Run from neuronify-core: `cargo run --release --example lab_gen`.
use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::elements::{spawn_connection, spawn_synapse, spawn_voltmeter, Element};
use neuronify_core::measurement::voltmeter::VoltmeterRange;
use neuronify_core::simulation::{fire_nearby, step_simulation};
use neuronify_core::*;

const REST: f64 = -70.0;
const THRESHOLD: f64 = -55.0;
const EPSP: f64 = 6.0; // mV, one synapse in the spatial and inhibition circuits
const SINGLE: f64 = 10.0; // mV, default synapse and the one-input circuits

struct Sim {
    world: World,
    time: f64,
    out: Entity,
    spikes: Vec<f64>,
}
impl Sim {
    fn new(world: World, out: Entity) -> Sim {
        Sim {
            world,
            time: 0.0,
            out,
            spikes: vec![],
        }
    }
    fn v(&self) -> f64 {
        self.world.get::<&LeakyDynamics>(self.out).unwrap().voltage * 1000.0
    }
    /// Runs ms of simulated time; returns (min, max) voltage seen.
    fn run(&mut self, ms: f64) -> (f64, f64) {
        let (mut lo, mut hi) = (self.v(), self.v());
        for _ in 0..(ms * 10.0).round() as usize {
            step_simulation(&mut self.world, &mut self.time, 1, None);
            let d = self.world.get::<&LeakyDynamics>(self.out).unwrap().clone();
            if d.time_since_fire == 0.0 {
                self.spikes.push(self.time * 1000.0);
            }
            lo = lo.min(d.voltage * 1000.0);
            hi = hi.max(d.voltage * 1000.0);
        }
        (lo, hi)
    }
    fn press(&mut self, button: Entity) {
        let p = self.world.get::<&Position>(button).unwrap().position;
        fire_nearby(&mut self.world, p);
    }
}

/// A voltmeter on `out`, with the range set to -90 to -40 mV so a 6 mV bump is easy to see.
fn vm(w: &mut World, out: Entity) {
    let z = w.get::<&Position>(out).unwrap().position;
    let m = spawn_voltmeter(w, out, Vec3::new(z.x, 0.0, z.z + 8.0));
    *w.get::<&mut VoltmeterRange>(m).unwrap() = VoltmeterRange {
        minimum: -90.0,
        maximum: -40.0,
    };
    *w.get::<&mut MeterSize>(m).unwrap() = MeterSize {
        width: 12.0,
        height: 6.0,
    };
}
fn neuron(w: &mut World, x: f32, z: f32, label: &str, ty: Element) -> Entity {
    let e = ty.spawn(w, Vec3::new(x, 0.0, z));
    w.get::<&mut Label>(e).unwrap().text = label.to_string();
    e
}
fn at(w: &mut World, el: Element, x: f32, z: f32) -> Entity {
    el.spawn(w, Vec3::new(x, 0.0, z))
}
fn note(w: &mut World, x: f32, z: f32, width: f32, height: f32, text: &str) {
    let e = at(w, Element::Note, x, z);
    w.get::<&mut Annotation>(e).unwrap().text = text.to_string();
    *w.get::<&mut NoteSize>(e).unwrap() = NoteSize { width, height };
}
fn syn(w: &mut World, from: Entity, to: Entity, nanoamps: f64) -> Entity {
    let s = spawn_synapse(w, from, to);
    w.get::<&mut CurrentSynapse>(s).unwrap().maximum_current = nanoamps * 1e-9;
    s
}
/// A touch sensor that fires every neuron in `to`.
fn button(w: &mut World, x: f32, z: f32, to: &[Entity]) -> Entity {
    let b = at(w, Element::TouchSensor, x, z);
    for &n in to {
        spawn_connection(w, b, n);
    }
    b
}

/// Peak EPSP (mV above rest) for a given maximum current, measured in the model.
fn epsp_peak(nanoamps: f64, inhibitory: bool) -> f64 {
    let mut w = World::new();
    let ty = if inhibitory {
        Element::InhibitoryNeuron
    } else {
        Element::ExcitatoryNeuron
    };
    let b = at(&mut w, Element::TouchSensor, 0.0, 0.0);
    let i = neuron(&mut w, 0.0, 6.0, "", ty);
    let o = neuron(&mut w, 0.0, 12.0, "", Element::ExcitatoryNeuron);
    spawn_connection(&mut w, b, i);
    syn(&mut w, i, o, nanoamps);
    let mut s = Sim::new(w, o);
    s.press(b);
    let (lo, hi) = s.run(80.0);
    if inhibitory {
        lo - REST
    } else {
        hi - REST
    }
}
/// Round to 0.05 nA the current that gives about `mv`.
fn current_for(mv: f64, inhibitory: bool) -> f64 {
    let (mut a, mut b) = (0.0, 10.0);
    for _ in 0..40 {
        let m = (a + b) / 2.0;
        if epsp_peak(m, inhibitory).abs() < mv {
            a = m
        } else {
            b = m
        }
    }
    (a * 20.0).round() / 20.0
}

struct Built {
    name: &'static str,
    world: World,
    out: Entity,
    buttons: Vec<(&'static str, Entity)>,
}

pub fn skeleton(w: &mut World) -> (Entity, Entity, Entity, Entity, Entity, Entity) {
    let a = neuron(w, 3.0, 8.0, "A", Element::ExcitatoryNeuron);
    let b = neuron(w, -3.0, 8.0, "B", Element::ExcitatoryNeuron);
    let out = neuron(w, 0.0, 16.0, "Output", Element::ExcitatoryNeuron);
    vm(w, out);
    let ba = button(w, 6.0, 0.0, &[a]);
    let bboth = button(w, 0.0, 0.0, &[a, b]);
    let bb = button(w, -6.0, 0.0, &[b]);
    note(w, 6.0, -6.0, 7.0, 1.6, "Input A");
    note(w, 0.0, -6.0, 7.0, 1.6, "Both");
    note(w, -6.0, -6.0, 7.0, 1.6, "Input B");
    (a, b, out, ba, bboth, bb)
}

fn build_all(one: f64, e6: f64, i7: f64) -> Vec<Built> {
    let mut all = vec![];
    // 1 threshold
    {
        let mut w = World::new();
        let out = neuron(&mut w, 0.0, 8.0, "Output", Element::ExcitatoryNeuron);
        vm(&mut w, out);
        let c = at(&mut w, Element::CurrentSource, 0.0, 2.0);
        w.get::<&mut CurrentClamp>(c).unwrap().current_output = 0.10e-9;
        spawn_connection(&mut w, c, out);
        note(&mut w, 10.0, 5.0, 18.0, 6.0, "Click the current source (the flat pad). Under Parameters, raise Current a little at a time and watch the voltmeter. Find the smallest current that makes the neuron fire.");
        all.push(Built {
            name: "1-threshold",
            world: w,
            out,
            buttons: vec![("source", c)],
        });
    }
    // 2 one-input
    {
        let mut w = World::new();
        let inp = neuron(&mut w, 0.0, 7.0, "Input", Element::ExcitatoryNeuron);
        let out = neuron(&mut w, 0.0, 14.0, "Output", Element::ExcitatoryNeuron);
        vm(&mut w, out);
        syn(&mut w, inp, out, one);
        let b = button(&mut w, 0.0, 1.5, &[inp]);
        note(&mut w, 10.0, 8.0, 18.0, 7.0, "Press the button once and watch the output. Then press it twice, quickly. Then press it twice with a long wait between the presses. How long can the wait be and the output still fires?");
        all.push(Built {
            name: "2-one-input",
            world: w,
            out,
            buttons: vec![("press", b)],
        });
    }
    // 3 steady rate
    {
        let mut w = World::new();
        let g = at(&mut w, Element::RegularSpikeGenerator, 0.0, 1.5);
        w.get::<&mut RegularSpikeGenerator>(g).unwrap().frequency = 10.0;
        let inp = neuron(&mut w, 0.0, 7.0, "Input", Element::ExcitatoryNeuron);
        let out = neuron(&mut w, 0.0, 14.0, "Output", Element::ExcitatoryNeuron);
        vm(&mut w, out);
        syn(&mut w, g, inp, 6.0);
        syn(&mut w, inp, out, one);
        note(&mut w, 10.0, 8.0, 18.0, 7.0, "The generator sends spikes at a steady rate. Click it and change Frequency under Parameters. Find the lowest frequency that makes the output fire.");
        all.push(Built {
            name: "3-steady-rate",
            world: w,
            out,
            buttons: vec![("generator", g)],
        });
    }
    // 4 spatial, 5 strength
    for strength in [false, true] {
        let mut w = World::new();
        let out = neuron(&mut w, 0.0, 19.0, "Output", Element::ExcitatoryNeuron);
        vm(&mut w, out);
        let mut ins = vec![];
        let mut bs = vec![];
        for (k, l) in ["A", "B", "C"].iter().enumerate() {
            let x = 6.0 - 6.0 * k as f32;
            let n = neuron(&mut w, x, 11.0, l, Element::ExcitatoryNeuron);
            syn(&mut w, n, out, e6);
            ins.push(n);
            bs.push((*l, button(&mut w, x, 3.0, &[n])));
            note(&mut w, x, -3.0, 7.0, 1.6, l);
        }
        let two = button(&mut w, 3.0, -6.0, &[ins[0], ins[1]]);
        let three = button(&mut w, -3.0, -6.0, &ins);
        note(&mut w, 3.0, -13.0, 10.0, 1.6, "A and B");
        note(&mut w, -3.0, -13.0, 10.0, 1.6, "A, B and C");
        let text = if strength {
            "Press each button, then A and B, then all three. Now click the line from A to Output and raise Maximum current until A alone makes the output fire."
        } else {
            "Press each button alone. Then press A and B. Then press all three. Which presses make the output fire?"
        };
        note(&mut w, 16.0, 8.0, 22.0, 6.0, text);
        bs.push(("two", two));
        bs.push(("three", three));
        all.push(Built {
            name: if strength {
                "5-synaptic-strength"
            } else {
                "4-spatial"
            },
            world: w,
            out,
            buttons: bs,
        });
    }
    // 6 inhibition
    {
        let mut w = World::new();
        let out = neuron(&mut w, 0.0, 19.0, "Output", Element::ExcitatoryNeuron);
        vm(&mut w, out);
        let mut es = vec![];
        for k in 0..4 {
            let n = neuron(
                &mut w,
                12.0 - 6.0 * k as f32,
                11.0,
                &format!("{}", k + 1),
                Element::ExcitatoryNeuron,
            );
            syn(&mut w, n, out, e6);
            es.push(n);
        }
        let inh = neuron(&mut w, -12.0, 11.0, "Inhibitory", Element::InhibitoryNeuron);
        syn(&mut w, inh, out, i7);
        let b_inh = button(&mut w, -12.0, 3.0, &[inh]);
        note(&mut w, -12.0, -3.0, 9.0, 1.6, "Inhibit");
        let b3 = button(&mut w, 9.0, -6.0, &es[..3]);
        let b4 = button(&mut w, 3.0, -6.0, &es);
        let b3i = button(&mut w, -3.0, -6.0, &[es[0], es[1], es[2], inh]);
        let b4i = button(&mut w, -9.0, -6.0, &[es[0], es[1], es[2], es[3], inh]);
        for (x, t) in [
            (9.0, "All three"),
            (3.0, "All four"),
            (-3.0, "All three plus inhibit"),
            (-9.0, "All four plus inhibit"),
        ] {
            note(&mut w, x, -15.0, 14.0, 1.6, t);
        }
        note(&mut w, 20.0, 9.0, 22.0, 6.0, "Press each button in turn and watch the output. Which presses push the voltage up, which pull it down, and which make the output fire?");
        all.push(Built {
            name: "6-inhibition",
            world: w,
            out,
            buttons: vec![
                ("inhibit", b_inh),
                ("three", b3),
                ("four", b4),
                ("three_inh", b3i),
                ("four_inh", b4i),
            ],
        });
    }
    // 7 refractory (optional)
    {
        let mut w = World::new();
        let g = at(&mut w, Element::RegularSpikeGenerator, 0.0, 1.5);
        w.get::<&mut RegularSpikeGenerator>(g).unwrap().frequency = 20.0;
        let inp = neuron(&mut w, 0.0, 7.0, "Input", Element::ExcitatoryNeuron);
        let out = neuron(&mut w, 0.0, 14.0, "Output", Element::ExcitatoryNeuron);
        w.get::<&mut LeakyDynamics>(out).unwrap().refractory_period = 0.01;
        vm(&mut w, out);
        syn(&mut w, g, inp, 6.0);
        syn(&mut w, inp, out, 6.0);
        note(&mut w, 10.0, 8.0, 18.0, 7.0, "Optional. The output needs a short rest after each spike. Click the generator and raise Frequency. Compare how many spikes go in with how many come out.");
        all.push(Built {
            name: "7-refractory-optional",
            world: w,
            out,
            buttons: vec![("generator", g)],
        });
    }
    all.push(logic("8-logic-challenge", one, 0));
    all.push(logic("solved-AND", one, 1));
    all.push(logic("solved-OR", one, 2));
    all.push(logic("solved-XOR", one, 3));
    all
}

/// kind 0 skeleton, 1 AND, 2 OR, 3 XOR
fn logic(name: &'static str, one: f64, kind: u8) -> Built {
    let mut w = World::new();
    let (a, b, out, ba, both, bb) = skeleton(&mut w);
    match kind {
        1 => {
            syn(&mut w, a, out, one);
            syn(&mut w, b, out, one);
        }
        2 => {
            syn(&mut w, a, out, 2.5);
            syn(&mut w, b, out, 2.5);
        }
        3 => {
            let i = neuron(&mut w, 0.0, 12.0, "Inhibitory", Element::InhibitoryNeuron);
            for n in [a, b] {
                let s = syn(&mut w, n, out, XOR.0);
                w.get::<&mut CurrentSynapse>(s).unwrap().delay = XOR.1;
                syn(&mut w, n, i, one);
            }
            syn(&mut w, i, out, XOR.2);
        }
        _ => {}
    }
    if kind == 0 {
        note(
            &mut w,
            14.0,
            8.0,
            20.0,
            7.0,
            "Take-home: connect A and B to Output so that the truth table is AND, then OR.",
        );
    }
    Built {
        name,
        world: w,
        out,
        buttons: vec![("A", ba), ("both", both), ("B", bb)],
    }
}

const XOR: (f64, f64, f64) = (2.5, 0.012, 10.0); // direct current, direct delay, inhibition current

fn main() {
    let one = current_for(SINGLE, false);
    let e6 = 0.8;
    let i7 = current_for(7.0, true);
    println!("SINGLE {SINGLE} mV -> {one} nA (peak {:.2}); EPSP {EPSP} -> {e6} nA (peak {:.2}); IPSP 7 -> {i7} nA (dip {:.2})",
        epsp_peak(one, false), epsp_peak(e6, false), epsp_peak(i7, true));
    println!("default synapse for new connections: {one} nA");
    let _ = (REST, THRESHOLD);
    verify(one, e6, i7);
    for b in build_all(one, e6, i7) {
        let bytes = serialization::save(&b.world, 0.0, DEFAULT_SPEED).unwrap();
        let dir = if b.name.starts_with("solved") {
            "examples/lab10/solutions"
        } else {
            "examples/lab10"
        };
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(format!("{dir}/{}.neuronify", b.name), bytes).unwrap();
        println!("wrote {}", b.name);
    }
}

fn fresh(name: &str, one: f64, e6: f64, i7: f64) -> (Sim, Vec<(&'static str, Entity)>) {
    let b = build_all(one, e6, i7)
        .into_iter()
        .find(|b| b.name == name)
        .unwrap();
    (Sim::new(b.world, b.out), b.buttons)
}
fn btn(bs: &[(&'static str, Entity)], n: &str) -> Entity {
    bs.iter().find(|b| b.0 == n).unwrap().1
}
fn spike_count(s: &Sim) -> usize {
    s.spikes.len()
}
fn verify(one: f64, e6: f64, i7: f64) {
    let f = |n: &str| fresh(n, one, e6, i7);
    // 1 threshold
    {
        let (mut s, _) = f("1-threshold");
        let (lo, hi) = s.run(300.0);
        println!(
            "1 current 0.10 nA: V settles {:.2} mV (min {lo:.1}, max {hi:.1}), spikes {}",
            s.v(),
            spike_count(&s)
        );
        let mut first = None;
        for c in (100..=300).map(|c| c as f64 * 0.001) {
            let (mut s, bs) = f("1-threshold");
            s.world
                .get::<&mut CurrentClamp>(btn(&bs, "source"))
                .unwrap()
                .current_output = c * 1e-9;
            s.run(500.0);
            if !s.spikes.is_empty() && first.is_none() {
                first = Some((c, s.spikes.len()));
            }
        }
        println!("1 first fires at {:?} nA (spikes in 500 ms)", first);
        let (mut s, bs) = f("1-threshold");
        s.world
            .get::<&mut CurrentClamp>(btn(&bs, "source"))
            .unwrap()
            .current_output = 0.3e-9;
        s.run(500.0);
        println!(
            "1 at 0.30 nA: {} spikes in 500 ms; at 2 nA default:",
            s.spikes.len()
        );
        let (mut s, bs) = f("1-threshold");
        s.world
            .get::<&mut CurrentClamp>(btn(&bs, "source"))
            .unwrap()
            .current_output = 2e-9;
        s.run(500.0);
        println!("   {} spikes", s.spikes.len());
    }
    // 2 one-input
    {
        let (mut s, bs) = f("2-one-input");
        s.press(btn(&bs, "press"));
        let mut peak = 0.0f64;
        let mut fade = None;
        for ms in 0..300 {
            let (_, hi) = s.run(1.0);
            peak = peak.max(hi);
            if ms > 10 && fade.is_none() && hi < REST + 0.5 {
                fade = Some(ms);
            }
        }
        println!("2 one press: peak {:.2} mV ({:.2} above rest), spikes {}, within 0.5 mV of rest after {:?} ms", peak, peak - REST, s.spikes.len(), fade);
        let mut longest = 0;
        for gap in 1..200 {
            let (mut s, bs) = f("2-one-input");
            let b = btn(&bs, "press");
            s.press(b);
            s.run(gap as f64);
            s.press(b);
            s.run(100.0);
            if !s.spikes.is_empty() {
                longest = gap;
            }
        }
        println!("2 longest gap that still fires: {longest} ms sim");
    }
    // 3 steady rate
    {
        let mut lowest = None;
        for hz in 5..400 {
            let (mut s, bs) = f("3-steady-rate");
            s.world
                .get::<&mut RegularSpikeGenerator>(btn(&bs, "generator"))
                .unwrap()
                .frequency = hz as f64;
            s.run(1000.0);
            if !s.spikes.is_empty() {
                lowest = Some(hz);
                break;
            }
        }
        let (mut s, _) = f("3-steady-rate");
        s.run(1000.0);
        println!(
            "3 lowest firing rate {:?} Hz; at 10 Hz spikes {}",
            lowest,
            s.spikes.len()
        );
        let (mut s, bs) = f("3-steady-rate");
        s.world
            .get::<&mut RegularSpikeGenerator>(btn(&bs, "generator"))
            .unwrap()
            .frequency = 100.0;
        s.run(1000.0);
        println!("3 at 100 Hz output spikes in 1 s: {}", s.spikes.len());
    }
    // 4 spatial and 6 inhibition
    for (name, keys) in [
        ("4-spatial", vec!["A", "B", "C", "two", "three"]),
        (
            "6-inhibition",
            vec!["inhibit", "three", "four", "three_inh", "four_inh"],
        ),
    ] {
        for k in keys {
            let (mut s, bs) = f(name);
            s.press(btn(&bs, k));
            let (lo, hi) = s.run(100.0);
            println!(
                "{name} {k}: min {lo:.1} max {hi:.1} mV, spikes {}",
                s.spikes.len()
            );
        }
    }
    // 5 strength
    {
        let mut found = None;
        for c in (1..=100).map(|c| c as f64 * 0.05) {
            let (mut s, bs) = f("5-synaptic-strength");
            let n = s
                .world
                .query::<&Label>()
                .iter()
                .find(|(_, l)| l.text == "A")
                .map(|(e, _)| e)
                .unwrap();
            let sy = s
                .world
                .query::<&Connection>()
                .with::<&CurrentSynapse>()
                .iter()
                .find(|(_, c)| c.from == n)
                .map(|(e, _)| e)
                .unwrap();
            s.world
                .get::<&mut CurrentSynapse>(sy)
                .unwrap()
                .maximum_current = c * 1e-9;
            s.press(btn(&bs, "A"));
            s.run(100.0);
            if !s.spikes.is_empty() {
                found = Some(c);
                break;
            }
        }
        println!(
            "5 A alone first fires at A to Output maximum current {:?} nA",
            found
        );
    }
    for n in ["solved-AND", "solved-OR", "solved-XOR", "8-logic-challenge"] {
        let mut row = String::new();
        for k in ["none", "A", "B", "both"] {
            let (mut s, bs) = f(n);
            if k != "none" {
                s.press(btn(&bs, k));
            }
            let (_, hi) = s.run(100.0);
            row += &format!(
                " {k}: {} (max {hi:.1})",
                if s.spikes.is_empty() { 0 } else { 1 }
            );
        }
        println!("{n}:{row}");
    }
    // 7 refractory
    for hz in [20.0, 50.0, 100.0, 200.0, 400.0] {
        let (mut s, bs) = f("7-refractory-optional");
        s.world
            .get::<&mut RegularSpikeGenerator>(btn(&bs, "generator"))
            .unwrap()
            .frequency = hz;
        s.run(1000.0);
        println!("7 {hz} Hz in: {} output spikes in 1 s", s.spikes.len());
    }
}
