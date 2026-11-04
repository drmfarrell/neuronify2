# Neuronify 2 for Cheyney University A&P I labs

This is a fork of [Neuronify 2](https://github.com/dragly/neuronify) (GPL-3.0) for the A&P I labs at Cheyney University of Pennsylvania.
It is based on upstream `main` at commit `5cc56981db7806506c736e344c531cc9a5fa696d`.
The source in this repository is the GPL source offer for the web build served from the `site` branch.

What this fork adds, all on the `cheyney-lab` branch:

- A URL parameter `?open=<file>` that opens a circuit with the editor showing (`?file=<file>` still opens it with the editor hidden).
- The Lab 10 circuits on synaptic summation in `neuronify-core/examples/lab10/`, listed on the landing page under "A&P Lab 10: synaptic summation".
  `neuronify-core/lab/gen.rs` builds them and checks their behaviour (`cargo run --release --example lab_gen`).
- A connection drawn in the editor between two neurons now starts at a maximum current of 1.3 nA (upstream: 6 nA), so one spike gives a bump of about 10 mV and does not fire the next neuron by itself. Bundled examples are not affected.
- `DEPLOY-WOLFNODE.md`: how to serve the prebuilt `site` branch. The `site` branch holds only the built static files, so the server needs no Rust.

The upstream README follows.

---

# Neuronify 2.0

Neuronify is an educational neural network app.
It gives a low entry point to simulation-based neuroscience.
Without prior computational experience, Neuronify makes it possible to work on neural simulations.

The original version of Neuronify is described in
[this paper published in eNeuro](https://doi.org/10.1523/ENEURO.0022-17.2017).
This repository contains the source code for Neuronify 2,
which is based on the same ideas and concepts as Neuronify 1,
but with new features and the ability to embed simulations into websites and online books.

Neuronify 1 was originally written in C++, while Neuronify 2 is written in Rust.

Each release on GitHub has builds for Linux, Windows and macOS,
and a web build that can be put in any folder on a web server.
To run Neuronify from the source code instead, run `cargo run --release -p neuronify`.

To run Neuronify in the browser,
install [trunk](https://trunkrs.dev) and run `trunk serve --port <port>` in the `neuronify` folder.
Adding `?file=examples/tutorial/2_leaks.neuronify` to the URL shows that file
the way it would look embedded on a web page, with the editor hidden.
It can be any legacy `.nfy` file or `.neuronify` save file the page can fetch.

Neuronify is licensed under the GNU General Public License version 3, see [LICENSE](LICENSE).
