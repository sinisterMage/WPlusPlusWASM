(async () => {
  try {
    console.log("🚀 Fetching and instantiating WASM...");

    const response = await fetch("ui.wasm");
    if (!response.ok) {
      throw new Error(`❌ Failed to fetch WASM: ${response.statusText}`);
    }

    const wasm = await WebAssembly.instantiateStreaming(response, {
      env: {
        memory: new WebAssembly.Memory({ initial: 10 }),

        drawRect: (x, y, w, h) => {
          console.log(`🟥 drawRect called with x=${x}, y=${y}, w=${w}, h=${h}`);

          const canvas = document.getElementById("screen");
          if (!canvas) {
            console.error("❌ Canvas element with id='screen' not found.");
            return;
          }

          const ctx = canvas.getContext("2d");
          if (!ctx) {
            console.error("❌ Failed to get 2D context from canvas.");
            return;
          }

          ctx.lineWidth = 4;
          ctx.strokeStyle = "red";
          ctx.fillStyle = "rgba(255, 0, 0, 0.4)";

          ctx.beginPath();
          ctx.rect(x, y, w, h);
          ctx.fill();
          ctx.stroke();
        },

        drawText: (x, y, ptr, len) => {
          console.log(`🔤 drawText called at x=${x}, y=${y}, ptr=${ptr}, len=${len}`);

          const memory = wasm.instance.exports.memory;
          if (!memory) {
            console.error("❌ WASM memory not exported.");
            return;
          }

          const bytes = new Uint8Array(memory.buffer, ptr, len);
          const str = new TextDecoder().decode(bytes);
          console.log(`🔤 Text to draw: "${str}"`);

          const canvas = document.getElementById("screen");
          if (!canvas) {
            console.error("❌ Canvas element with id='screen' not found.");
            return;
          }

          const ctx = canvas.getContext("2d");
          if (!ctx) {
            console.error("❌ Failed to get 2D context from canvas.");
            return;
          }

          ctx.fillStyle = "green";
          ctx.font = "16px sans-serif";
          ctx.fillText(str, x, y);
        },

        gc_alloc: () => {
          console.log("🧠 gc_alloc stub called");
          return 1024;
        },
        add_root: () => {
          console.log("🌱 add_root stub called");
        },
        gc_tick: () => {
          console.log("🧹 gc_tick stub called");
        },
      },
    });

    console.log("✅ WASM instantiated successfully. Calling `run()`...");
    wasm.instance.exports.run?.();
  } catch (err) {
    console.error("🔥 Error during WASM loading or execution:", err);
  }
})();
