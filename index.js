import init from "./pkg/alg_viz.js";
// TODO: fix the import using
// https://stackoverflow.com/questions/61986932/how-to-pass-a-string-from-js-to-wasm-generated-through-rust-using-wasm-bindgen-w
init()
  .then((wasm) => {
    const canvas = document.getElementById("canvas");
    const context = canvas.getContext("2d");
    var delay = document.getElementById("delay");

    var timer = null;
    var play = false;

    document.getElementById("string").addEventListener("change", (event) => {
      window.clearTimeout(timer);
      timer = null;
      wasm.reset();
    });

    document.getElementById("query").addEventListener("change", (event) => {
      window.clearTimeout(timer);
      timer = null;
      wasm.reset();
    });

    document.getElementById("algorithm").addEventListener("change", (event) => {
      window.clearTimeout(timer);
      timer = null;
      wasm.reset();
    });

    document.getElementById("prev").addEventListener("click", (event) => {
      wasm.prev();
    });

    document.getElementById("next").addEventListener("click", (event) => {
      wasm.next();
    });

    function maketimer() {
      timer = window.setTimeout(() => {
        wasm.next();
        maketimer();
      }, delay.value * 1000);
    }

    function faster() {
      delay.value /= 1.5;
    }

    function slower() {
      delay.value *= 1.5;
    }

    function pauseplay() {
      if (play) {
        play = false;
        window.clearTimeout(timer);
        timer = null;
      } else {
        play = true;
        maketimer();
      }
    }

    document.getElementById("faster").addEventListener("click", faster);
    document.getElementById("slower").addEventListener("click", slower);
    document.getElementById("pauseplay").addEventListener("click", pauseplay);

    wasm.reset();

    document.onkeydown = function (e) {
      switch (e.keyCode) {
        case 8: // backspace
        case 37: // left
          wasm.prev();
          return false;
        case 32: // space
        case 39: // right
          wasm.next();
          return false;
        case 38: // up
        case 70: // f
        case 187: // +
          faster();
          return false;
        case 40: // down
        case 83: // s
        case 189: // -
          slower();
          return false;
        case 13: // return
        case 80: // p
          pauseplay();
          return false;
      }
      return true;
    };
  })
  .catch(console.error);
