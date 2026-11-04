# Deploying Neuronify (Cheyney lab fork) on wolfnode

This brief is for the Claude Code agent on the wolfnode server.

## Goal

Serve the `site` branch of https://github.com/drmfarrell/neuronify2 as plain static files at
**https://neuronify.wolfnode.org/**.
It is a new Cloudflare-proxied subdomain, set up the same way as the existing `chemsim.wolfnode.org`.
Look at how `chemsim` is served on this box (web server, site folder, certificate, Cloudflare record) and follow the same conventions.
Do not invent paths or credentials. The site needs no Rust, no build step and no server-side code.

## Get the files

```
git clone -b site --single-branch https://github.com/drmfarrell/neuronify2.git neuronify-site
```

Put the checkout where this box keeps its static sites, and point the web server's document root for `neuronify.wolfnode.org` at the checkout folder (the folder that contains `index.html`).
The `.git` folder must not be served: deny `/.git/` in the server config.

## Server settings

- `.wasm` must be sent as `Content-Type: application/wasm`. Browsers refuse to start the app otherwise. Most servers do this by default (nginx `mime.types` has it, Caddy does too). Check with the list below.
- COOP and COEP headers are **not** needed. The app runs single-threaded and does not use SharedArrayBuffer.
- No SPA fallback: a missing file should return 404, not `index.html`.
- Compression (gzip or brotli) for `.wasm`, `.js` and `.html` is welcome: the `.wasm` file is 8.5 MB uncompressed.
- Caching: `index.html` and everything under `examples/` keep the same names when they change, so send `Cache-Control: no-cache` (revalidate) for them.
  The `.js`, `.wasm` and loader files have a hash in their names, so they can be cached for a long time (`max-age=31536000, immutable`).
  Behind Cloudflare, make sure `index.html` and `examples/*` are not cached for long there either (a Cache Rule or "respect origin headers"), or purge after an update.
- The app uses relative URLs, so it also works from a subpath. It works at the root of the subdomain.

## Update

```
cd neuronify-site && git pull
```

Nothing else. If Cloudflare caches the old `index.html`, purge it.

## Addresses

Site root (landing page with all examples): https://neuronify.wolfnode.org/

A lab circuit opens in the editor with `?open=` (relative to the site root):

| Circuit | Relative to the site root |
|---|---|
| 1 Threshold | `/?open=examples/lab10/1-threshold.neuronify` |
| 2 One input, two presses | `/?open=examples/lab10/2-one-input.neuronify` |
| 3 Steady rate | `/?open=examples/lab10/3-steady-rate.neuronify` |
| 4 Three inputs at once | `/?open=examples/lab10/4-spatial.neuronify` |
| 5 Synaptic strength | `/?open=examples/lab10/5-synaptic-strength.neuronify` |
| 6 Inhibition | `/?open=examples/lab10/6-inhibition.neuronify` |
| 7 Refractory period (optional) | `/?open=examples/lab10/7-refractory-optional.neuronify` |
| 8 Take-home: logic challenge | `/?open=examples/lab10/8-logic-challenge.neuronify` |

Put `https://neuronify.wolfnode.org` in front of each.
The answer-key circuits are not in the public repository.

## Verify (run these on the box)

```
curl -sI https://neuronify.wolfnode.org/ | head -5                     # 200, text/html
W=$(grep -o 'neuronify-[0-9a-f]*_bg.wasm' neuronify-site/index.html | head -1)
curl -sI https://neuronify.wolfnode.org/$W | grep -i '^content-type'   # application/wasm
for n in 1-threshold 2-one-input 3-steady-rate 4-spatial 5-synaptic-strength 6-inhibition 7-refractory-optional 8-logic-challenge; do
  curl -s -o /dev/null -w "%{http_code} $n\n" https://neuronify.wolfnode.org/examples/lab10/$n.neuronify
done                                                                    # eight lines starting with 200
```

Then open `https://neuronify.wolfnode.org/?open=examples/lab10/2-one-input.neuronify` in a real browser (not headless). You should see a toolbar on the left with Edit selected, a note, three circles and a voltmeter box. Click the View button, then click the orange touch button: the voltmeter trace should show a small bump. Report the results and anything that differs from the existing `chemsim` setup.

The browser needs WebGPU or WebGL2, which current Chrome, Edge, Firefox and Safari have.

Address parameters: `?open=<file>` starts in the Stimulate tool, so a first press on a touch sensor fires a neuron. Add `&mode=edit` or `&mode=view` to start in another tool (the take-home logic challenge uses `&mode=edit`).

## For the server agent: make Cloudflare cache the app files

Measured on 2026-11-04 from the instructor's box: the `.wasm` file is served gzip (3.4 MB) with `cache-control: public, max-age=31536000, immutable`, but the response says `cf-cache-status: DYNAMIC`. Cloudflare does not cache `.wasm` by default (it is not in its default cached extensions), so every student download comes from the origin at about 100 to 170 KB/s, which is 20 to 34 seconds each. A room of 25 opening the page together would share that speed.

This needs Cloudflare dashboard or API access; the repository cannot fix it.

1. Create a Cache Rule for host `neuronify.wolfnode.org`: for paths ending in `.wasm` or `.js`, set "Eligible for cache" and "Respect origin TTL" (the files carry a one-year immutable TTL because their names contain a hash).
2. Keep `index.html` short-lived so that a new build is seen after a pull: either leave it uncached, or give it an edge TTL of a few minutes.
3. Check with `curl -sI https://neuronify.wolfnode.org/<the .wasm file named in index.html>` twice. The first request may show `cf-cache-status: MISS`; the second must show `cf-cache-status: HIT`.
