#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
mkdir -p docs

for f in ./target/wasm32-unknown-unknown/release/*.wasm; do
   wasm-bindgen $f --out-dir ./docs --no-modules --browser
done

for f in ./docs/*.wasm; do
  wasm=${f/\.\/docs\//""}
  wasm=${wasm/_bg/""}
  wasm=${wasm/.wasm/""}
  file=$(cat <<-END
      <html>
      <head>
          <title>LearnOpenGL-Notan - $wasm</title>
          <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
          <meta http-equiv="X-UA-Compatible" content="IE=edge">
          <meta name="viewport" content="minimal-ui, width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
          <meta name="apple-mobile-web-app-capable" content="yes">

          <style>
              html, body {
                  width: 100%;
                  height: 100%;
                  padding: 0;
                  margin: 0;
                  background-color: #252526;
              }

              * {
                  outline: none;
              }

              div#container {
                  width: 100%;
                  display: flex;
                  align-items: center;
                  justify-content: center;
              }

              div#source-code {
                  color: #ffffff;
                  margin: auto;
                  text-align: center;
                  padding: 30px;
              }

              div#source-code a {
                  text-decoration: none;
                  color: #f3d080;
              }
          </style>
      </head>
      <body>
      <script src="./$wasm.js"></script>
      <script>
            window.addEventListener("load", async () => {
                  let wasm = await wasm_bindgen("./${wasm}_bg.wasm");
            });
      </script>
      <div id="container">
          <canvas id="notan_canvas"></canvas>
      </div>
      </body>
      </html>
  )

    echo $file > "./docs/$wasm.html"
done