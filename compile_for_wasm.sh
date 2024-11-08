rm -rf ./target/wasm-mode/*
cargo build --release  --target wasm32-unknown-unknown
mv target/wasm32-unknown-unknown/release/fartvy.wasm ./target/wasm-mode/fartvy.wasm
wasm-bindgen --out-dir ./target/wasm-mode/ --target web ./target/wasm-mode/fartvy.wasm 

echo '
<html>
  <head>
    <meta charset="UTF-8" />

    <style>
      body,
      html {
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
        overflow: hidden;
      }
      .canvas {
        display: block;
        width: 100%;
        height: 100%;
      }

      @-webkit-keyframes spin {
        0% {
          -webkit-transform: rotate(0deg);
        }

        100% {
          -webkit-transform: rotate(360deg);
        }
      }

      @keyframes spin {
        0% {
          transform: rotate(0deg);
        }

        100% {
          transform: rotate(360deg);
        }
      }
    </style>
  </head>

  <body>
    <div class="loader"></div>

    <script>
      // the following function keeps track of all AudioContexts and resumes them on the first user

      // interaction with the page. If the function is called and all contexts are already running,

      // it will remove itself from all event listeners.

      (function () {
        // An array of all contexts to resume on the page

        const audioContextList = [];

        // An array of various user interaction events we should listen for

        const userInputEventNames = [
          "click",

          "contextmenu",

          "auxclick",

          "dblclick",

          "mousedown",

          "mouseup",

          "pointerup",

          "touchend",

          "keydown",

          "keyup",
        ];

        // A proxy object to intercept AudioContexts and

        // add them to the array for tracking and resuming later

        self.AudioContext = new Proxy(self.AudioContext, {
          construct(target, args) {
            const result = new target(...args);

            audioContextList.push(result);

            return result;
          },
        });

        // To resume all AudioContexts being tracked

        function resumeAllContexts(_event) {
          let count = 0;

          audioContextList.forEach((context) => {
            if (context.state !== "running") {
              context.resume();
            } else {
              count++;
            }
          });

          // If all the AudioContexts have now resumed then we unbind all

          // the event listeners from the page to prevent unnecessary resume attempts

          // Checking count > 0 ensures that the user interaction happens AFTER the game started up

          if (count > 0 && count === audioContextList.length) {
            userInputEventNames.forEach((eventName) => {
              document.removeEventListener(eventName, resumeAllContexts);
            });
          }
        }

        // We bind the resume function for each user interaction

        // event on the page

        userInputEventNames.forEach((eventName) => {
          document.addEventListener(eventName, resumeAllContexts);
        });
      })();
    </script>

    <script type="module">
      import init from "./fartvy.js";

      init();
    </script>
  </body>
</html>

' > ./target/wasm-mode/index.html