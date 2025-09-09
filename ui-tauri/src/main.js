const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let pttButtonEl;
let pttLogEl;
let aiButtonEl;
let aiLogEl;

const keyMap = {
          "KeyQ": ["Q", "#"],
          "KeyW": ["W", "1"],
          "KeyE": ["E", "2"],
          "KeyR": ["R", "3"],
          "KeyT": ["T", "("],
          "KeyY": ["Y", ")"],
          "KeyU": ["U", "_"],
          "KeyI": ["I", "-"],
          "KeyO": ["O", "+"],
          "KeyP": ["P", "@"],
          "KeyA": ["A", "*"],
          "KeyS": ["S", "4"],
          "KeyD": ["D", "5"],
          "KeyF": ["F", "6"],
          "KeyG": ["G", "/"],
          "KeyH": ["H", ":"],
          "KeyJ": ["J", ";"],
          "KeyK": ["K", "'"],
          "KeyL": ["L", '"'],
          "KeyZ": ["Z", "7"],
          "KeyX": ["X", "8"],
          "KeyC": ["C", "9"],
          "KeyV": ["V", "?"],
          "KeyB": ["B", "!"],
          "KeyN": ["N", ","],
          "KeyM": ["M", "."],
          "Digit4": ["$", "🔊"],
          "MetaLeft": ["🎤︎", "0"],
};

function remap(sym) {
  for (let className in keyMap) {
    let [normalVal, symVal] = keyMap[className];
    document.querySelector("." + className).innerText = (sym)? symVal : normalVal;
  }
}

async function start() {
  await invoke("start");
}

async function ptt_button_press(name) {
  await invoke("ptt_button_press", { name });
}

async function ai_button_press(name) {
  await invoke("ai_button_press", { name });
}

async function handle_ptt(e) {
  if (e.payload === "Idle") {
    pttButtonEl.innerText = "PTT";
    pttButtonEl.disabled = false;
  } else if (e.payload === "Recording") {
    pttButtonEl.innerText = "RECORDING";
    pttButtonEl.disabled = false;
  } else if (e.payload === "Playing") {
    pttButtonEl.innerText = "PLAYING";
    pttButtonEl.disabled = true;
  }
}

async function handle_screen(e) {
  let { left, right, top, bottom, data } = e.payload;

  const screen = document.getElementById("screen");
  const ctx = screen.getContext("2d");

  const width = right - left;
  const height = bottom - top;
  const imageData = ctx.createImageData(width, height);

  if (data.length != imageData.data.length) {
    console.log(`malformed command ${data.length} != ${imageData.data.length}`);
    return;
  }

  for (let i = 0; i < imageData.data.length; i ++) {
    imageData.data[i] = data[i];
  }
  
  ctx.putImageData(imageData, left, top);
}

async function handle_led(e) {
  console.log("handle_led", e);

  let {name, r, g, b} = e.payload;

  const led = document.getElementById(name);
  led.style.background = `rgb(${r}, ${g}, ${b})`;
}

async function handle_events() {
  const ptt_state = listen('PttState', handle_ptt);
  const screen = listen('Screen', handle_screen);
  const led = listen('LED', handle_led);
  return Promise.all([ptt_state, screen, led])
}

window.addEventListener("DOMContentLoaded", async () => {
  console.log("content loaded");

  // Configure the PTT button
  pttButtonEl = document.querySelector("#ptt-button");

  pttButtonEl.addEventListener("mousedown", (e) => {
    ptt_button_press("mousedown");
  });

  pttButtonEl.addEventListener("mouseup", (e) => {
    ptt_button_press("mouseup");
  });

  // Configure the AI button
  aiButtonEl = document.querySelector("#ai-button");

  aiButtonEl.addEventListener("mousedown", (e) => {
    ai_button_press("mousedown");
  });

  aiButtonEl.addEventListener("mouseup", (e) => {
    ai_button_press("mouseup");
  });

  let asClass = (code) => {
    return "." + code;
  }

  let sym = false;
  let l_shift = false;
  let r_shift = false;

  // Capture keyboard events
  document.addEventListener("keydown", (e) => {
    console.log("code:", e.code);

    if (e.code == "BracketLeft") {
      ptt_button_press("mousedown");
      return;
    } 

    if (e.code == "BracketRight") {
      ai_button_press("mousedown");
      return;
    }

    let code = e.keyCode;
    if (e.code == "AltRight") {
      sym = true;

      // Both alt keys have the same code othewise, and this doesn't collide
      code += 1; 
    }

    if (e.code == "ShiftLeft") {
      l_shift = true;
    }

    if (e.code == "ShiftRight") {
      r_shift = true;

      // Both shift keys have the same code othewise, and this doesn't collide
      code += 1; 
    }

    if (sym) {
      remap(true);
    }

    let value = e.code;
    if (value in keyMap) {
      if (sym) {
        value = keyMap[value][1];
      } else {
        value = keyMap[value][0];
      }

      if (l_shift || r_shift) {
        value = value.toUpperCase();
      } else {
        value = value.toLowerCase();
      }
    }

    document.querySelector(asClass(e.code)).classList.add("active");
    invoke("keydown", { code, value });
  });

  document.addEventListener("keyup", (e) => {
    if (e.code == "BracketLeft") {
      ptt_button_press("mouseup");
      return;
    } 

    if (e.code == "BracketRight") {
      ai_button_press("mouseup");
      return;
    }
    
    let code = e.keyCode;

    if (e.code == "AltRight") {
      sym = false;
      remap(false);

      // Both alt keys have the same code othewise, and this doesn't collide
      code += 1; 
    }

    if (e.code == "ShiftLeft") {
      l_shift = false;
    }

    if (e.code == "ShiftRight") {
      r_shift = false;

      // Both shift keys have the same code othewise, and this doesn't collide
      code += 1; 
    }

    document.querySelector(asClass(e.code)).classList.remove("active");
    invoke("keyup", { code });
  });

  // Await events
  await handle_events();

  // Map the keyboard to normal keycaps
  remap(false);

  // Notify the backend that the UI has started
  start();
});
