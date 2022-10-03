// @flow
/**
 * LICENSE ...
 * Author: ...
 */
import React, { useEffect, useMemo, useState } from "react";
import init, { render } from "../rust/pkg/main";
import wasm from "base64-inline-loader!../rust/pkg/main_bg.wasm";
import generateVariables, { getPerf, height, width } from "./variables";

function decode(dataURI) {
  const binaryString = atob(dataURI.split(",")[1]);
  var bytes = new Uint8Array(binaryString.length);
  for (var i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}
let wasmLoaded = false;
const promiseOfLoad = init(decode(wasm)).then(() => {
  wasmLoaded = true;
});

const svgSize = [width, height];
const MAX = 4096;
const ratio = svgSize[0] / svgSize[1];
const svgMMSize = svgSize.map((s) => s + "mm");

let adaptiveSvgWidth = (width) => Math.max(64, Math.ceil(width / 64) * 64);

const Main = ({ width, height, random }) => {
  const dpr = window.devicePixelRatio || 1;
  let W = width;
  let H = height;
  H = Math.min(H, W / ratio);
  W = Math.min(W, H * ratio);
  W = Math.floor(W);
  H = Math.floor(H);
  let w = Math.min(MAX, dpr * W);
  let h = Math.min(MAX, dpr * H);
  h = Math.min(h, w / ratio);
  w = Math.min(w, h * ratio);
  w = Math.floor(w);
  h = Math.floor(h);
  const svgW = adaptiveSvgWidth(w);
  const widthPx = svgW + "px";
  const heightPx = Math.floor(svgW / ratio) + "px";

  const [loaded, setLoaded] = useState(wasmLoaded);
  const variables = useVariables({ random });

  useEffect(() => {
    if (!loaded) promiseOfLoad.then(() => setLoaded(true));
  }, [loaded]);

  const svg = useMemo(() => {
    if (!loaded) return "";
    let prev = Date.now();
    const result = render(variables.opts);
    console.log(
      "svg calc time = " +
        (Date.now() - prev) +
        "ms – " +
        (result.length / (1024 * 1024)).toFixed(3) +
        " Mb"
    );
    window.$fxhashFeatures = generateVariables.inferProps(variables, result);
    if (console && console.table) {
      console.table(window.$fxhashFeatures);
      const p = getPerf(result);
      if (p) {
        console.table(p.per_label);
      }
    }
    return result;
  }, [variables.opts, loaded]);

  const renderedSVG = useMemo(
    () =>
      "data:image/svg+xml;base64," +
      btoa(svg.replace(svgMMSize[1], heightPx).replace(svgMMSize[0], widthPx)),
    [svg, widthPx, heightPx]
  );

  return (
    <div
      style={{
        width,
        height,
        position: "relative",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div style={{ position: "relative", width: W, height: H }}>
        <div
          style={{
            zIndex: 1,
            position: "relative",
            pointerEvents: "none",
            background: "white",
          }}
        >
          <img width="100%" src={renderedSVG} />
        </div>
        <Downloadable
          svg={svg}
          layers={variables.layers}
          background={variables.background}
        />
      </div>
    </div>
  );
};

const dlStyle = {
  opacity: 0,
  width: "100%",
  height: "100%",
  zIndex: 0,
  position: "absolute",
  top: 0,
  left: 0,
};
function Downloadable({ svg, layers, background }) {
  const [uri, setURI] = useState(null);
  useEffect(() => {
    const timeout = setTimeout(() => {
      let svgOut = svg
        .replace(
          "background:" + background.placeholder,
          `background:${
            "rgb(" +
            background.rgb.map((c) => Math.floor(c * 255)).join(",") +
            ")"
          }`
        )
        .replace(/opacity="[^"]*"/g, 'style="mix-blend-mode: multiply"');

      layers.forEach((l) => {
        svgOut = svgOut.replace(
          l.search,
          "rgb(" + l.rgb.map((n) => Math.round(n * 255)).join(",") + ")"
        );
      });

      setURI("data:image/svg+xml;base64," + btoa(svgOut));
    }, 500);
    return () => clearTimeout(timeout);
  }, [svg, layers, background]);

  return <img style={dlStyle} src={uri} />;
}

function useVariables({ random }) {
  return useMemo(
    () =>
      generateVariables(
        random,
        window.fxhash,
        new URLSearchParams(window.location.search).get("debug") === "1"
      ),
    []
  );
}

export default Main;
