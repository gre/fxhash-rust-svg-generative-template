/**
 * LICENSE: ...
 * Author: ...
 */

const width = 100;
const height = 150;
const pad = 5;

module.exports.width = width;
module.exports.height = height;
module.exports.pad = pad;

module.exports = function generateVariables(random, hash, debug = false) {
  let background = {
    placeholder: "white",
    rgb: [1, 1, 1],
  };
  let layers = [
    { name: "black", search: /#0FF/g, rgb: [0, 0, 0] },
    { name: "red", search: /#F0F/g, rgb: [1, 0, 0] },
    { name: "blue", search: /#F0F/g, rgb: [0, 0, 1] },
  ];

  const opts = {
    layer1_name: layers[0].name,
    layer2_name: layers[1].name,
    layer3_name: layers[2].name,
    width,
    height,
    pad,
    hash,
    debug,
  };

  // eslint-disable-next-line no-undef
  if (process.env.NODE_ENV !== "production" && typeof window !== "undefined") {
    console.log(window.fxhash);
    Object.keys(opts).forEach((key) => console.log(key + " =", opts[key]));
  }

  return {
    opts,
    layers,
    background,
  };
};

module.exports.inferProps = function inferProps(variables, svg) {
  const m = svg.match("data-traits='([^']+)'");
  const props = JSON.parse(m[1]);
  for (let k in props) {
    if (!props[k]) {
      delete props[k];
    }
  }
  return props;
};

module.exports.getPerf = function getPerf(svg) {
  const m = svg.match("data-perf='([^']+)'");
  if (!m) return;
  return JSON.parse(m[1]);
};
