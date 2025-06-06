const memory = new WebAssembly.Memory({ initial: 1 });
let heap = new Uint8Array(memory.buffer);
let nextAlloc = 0;

function gc_alloc(size) {
    const addr = nextAlloc;
    nextAlloc += size;

    // You can add metadata tracking here
    console.log(`🧠 Allocated ${size} bytes at 0x${addr.toString(16)}`);
    return addr;
}

function drawRect(x, y, w, h) {
    const canvas = document.getElementById("screen");
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "blue";
    ctx.fillRect(x, y, w, h);
    console.log(`🟦 Drew rect: (${x}, ${y}, ${w}, ${h})`);
}

async function runWasm() {
    const response = await fetch("ui.wasm");
    const bytes = await response.arrayBuffer();

    const module = await WebAssembly.instantiate(bytes, {
        env: {
            drawRect,
            gc_alloc,
            memory,
        },
    });

    module.instance.exports.run();
}

window.onload = runWasm;
