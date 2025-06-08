(async () => {
  const wasm = await WebAssembly.instantiateStreaming(fetch("out.wasm"), {
    env: {
      memory: new WebAssembly.Memory({ initial: 10 }),
      drawRect: (x, y, w, h) => {
        const ctx = document.getElementById("screen").getContext("2d");
        ctx.strokeStyle = "red";
        ctx.strokeRect(x, y, w, h);
      },
      drawText: (x, y, ptr, len) => {
        // Memory access for string
        const bytes = new Uint8Array(wasm.instance.exports.memory.buffer, ptr, len);
        const str = new TextDecoder().decode(bytes);
        const ctx = document.getElementById("screen").getContext("2d");
        ctx.fillStyle = "green";
        ctx.fillText(str, x, y);
      },
      gc_alloc: () => 1024, // stub
      add_root: () => {},
      gc_tick: () => {},
    },
  });

  wasm.instance.exports.run(); // run main
})();
