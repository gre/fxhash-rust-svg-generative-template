/**
 * LICENSE ...
 * Author: ...
 */
mod utils;
use instant::Instant;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use svg::node::element::path::Data;
use svg::node::element::{Group, Path};
use svg::Document;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn render(val: &JsValue) -> String {
  let opts = val.into_serde().unwrap();
  let doc = art(&opts);
  let str = doc.to_string();
  return str;
}

#[derive(Deserialize)]
pub struct Opts {
  pub hash: String,
  pub width: f64,
  pub height: f64,
  pub pad: f64,
  pub layer1_name: String,
  pub layer2_name: String,
  pub layer3_name: String,
  pub debug: bool,
}

pub fn art(opts: &Opts) -> Document {
  let width = opts.width;
  let height = opts.height;
  let pad = opts.pad;

  let mut perf = PerfRecords::start(opts.debug);
  // constants
  let size = 1.0;

  // Now, we'll determine most of the rng properties
  let mut rng = rng_from_fxhash(opts.hash.clone());
  let dots_count = rng.gen_range(50, 200);

  // this is where we aggregate our paths
  let mut layer1: Vec<Vec<Vec<(f64, f64)>>> = Vec::new();
  let mut layer2: Vec<Vec<Vec<(f64, f64)>>> = Vec::new();
  let mut layer3: Vec<Vec<Vec<(f64, f64)>>> = Vec::new();

  // Implement your art
  perf.span("dots");
  let mut dots = Vec::new();
  for _i in 0..dots_count {
    let p = (
      rng.gen_range(pad, width - pad),
      rng.gen_range(pad, height - pad),
    );
    dots.push(vec![
      p,
      (p.0 + size, p.1),
      (p.0 + 0.5 * size, p.1 + 0.5 * size),
      p,
    ])
  }
  layer1.push(dots);
  perf.span_end("dots");

  // Generate the svg
  perf.span("svg");
  let (layers, inks) = make_layers(vec![
    ("#0FF", opts.layer1_name.clone(), layer1.concat()),
    ("#F0F", opts.layer2_name.clone(), layer2.concat()),
    ("#FF0", opts.layer3_name.clone(), layer3.concat()),
  ]);
  perf.span_end("svg");

  // add the traits
  let mut traits = Map::new();
  traits.insert(String::from("Dots Count"), json!(dots_count));

  let mut document = svg::Document::new()
    .set("data-hash", opts.hash.to_string())
    .set("data-traits", Value::Object(traits).to_string())
    .set("viewBox", (0, 0, width, height))
    .set("width", format!("{}mm", width))
    .set("height", format!("{}mm", height))
    .set("style", "background:white")
    .set(
      "xmlns:inkscape",
      "http://www.inkscape.org/namespaces/inkscape",
    )
    .set("xmlns", "http://www.w3.org/2000/svg");
  if opts.debug {
    document = document.set("data-perf", json!(perf.end()).to_string());
  }
  for l in layers {
    document = document.add(l);
  }
  document
}

// render helper

#[inline]
fn significant_str(f: f64) -> f64 {
  (f * 100.0).floor() / 100.0
}

fn render_route(data: Data, route: Vec<(f64, f64)>) -> Data {
  if route.len() == 0 {
    return data;
  }
  let first_p = route[0];
  let mut d =
    data.move_to((significant_str(first_p.0), significant_str(first_p.1)));
  for p in route {
    d = d.line_to((significant_str(p.0), significant_str(p.1)));
  }
  return d;
}

fn rng_from_fxhash(hash: String) -> impl Rng {
  let mut bs = [0; 32];
  bs58::decode(hash.chars().skip(2).take(43).collect::<String>())
    .into(&mut bs)
    .unwrap();
  let rng = StdRng::from_seed(bs);
  return rng;
}

fn make_layers(
  data: Vec<(&str, String, Vec<Vec<(f64, f64)>>)>,
) -> (Vec<Group>, Vec<String>) {
  let mut inks = Vec::new();
  let layers: Vec<Group> = data
    .iter()
    .filter(|(_color, _label, routes)| routes.len() > 0)
    .map(|(color, label, routes)| {
      inks.push(label.clone());
      let mut l = Group::new()
        .set("inkscape:groupmode", "layer")
        .set("inkscape:label", label.clone())
        .set("fill", "none")
        .set("stroke", color.clone())
        .set("stroke-linecap", "round")
        .set("stroke-width", 0.35);
      let opacity: f64 = 0.6;
      let opdiff = 0.15 / (routes.len() as f64);
      let mut trace = 0f64;
      for route in routes.clone() {
        trace += 1f64;
        let data = render_route(Data::new(), route);
        l = l.add(
          Path::new()
            .set(
              "opacity",
              (1000. * (opacity - trace * opdiff)).floor() / 1000.0,
            )
            .set("d", data),
        );
      }
      l
    })
    .collect();
  // remove inks that have no paths at all
  inks.sort();
  if inks.len() == 2 && inks[0].eq(&inks[1]) {
    inks.remove(1);
  }
  (layers, inks)
}

// PERFORMANCE HELPERS
struct Span {
  label: String,
  start: Instant,
  stop: Instant,
}
struct PerfRecords {
  debug: bool,
  started: HashMap<String, Instant>,
  spans: Vec<Span>,
}
struct PerfResult {
  per_label: HashMap<String, f64>,
}
impl PerfRecords {
  /**
   * let mut perf = PerfRecords::start();
   */
  pub fn start(debug: bool) -> Self {
    let mut r = PerfRecords {
      debug,
      started: HashMap::new(),
      spans: Vec::new(),
    };
    r.span("total");
    r
  }
  /**
   * perf.span("calc_circles");
   */
  pub fn span(self: &mut Self, s: &str) {
    if self.debug {
      self.started.insert(String::from(s), Instant::now());
    }
  }
  /**
   * perf.span_end("calc_circles");
   */
  pub fn span_end(self: &mut Self, s: &str) {
    if self.debug {
      let label = String::from(s);
      if let Some(&start) = self.started.get(&label) {
        self.spans.push(Span {
          label,
          start,
          stop: Instant::now(),
        });
      }
    }
  }
  /**
   * let perf_res = perf.end();
   */
  pub fn end(self: &mut Self) -> PerfResult {
    let mut per_label = HashMap::new();
    if self.debug {
      self.span_end("total");
      self.spans.iter().for_each(|span| {
        let maybe_time = per_label.get(&span.label).unwrap_or(&0.);
        per_label.insert(
          span.label.clone(),
          maybe_time + span.stop.duration_since(span.start).as_secs_f64(),
        );
      });
    }
    PerfResult { per_label }
  }
}

impl Serialize for PerfResult {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_struct("Perf", 1)?;
    state.serialize_field("per_label", &self.per_label)?;
    state.end()
  }
}
