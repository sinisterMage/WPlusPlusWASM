const memory = new WebAssembly.Memory({ initial: 1 });
let heap = new Uint8Array(memory.buffer);
let nextAlloc = 0;

// === GC bump allocator ===
function gc_alloc(size) {
    const addr = nextAlloc;
    nextAlloc += size;

    console.log(`🧠 Allocated ${size} bytes at 0x${addr.toString(16)}`);
    return addr;
}

// === Canvas draw ===
function drawRect(x, y, w, h) {
    const canvas = document.getElementById("screen");
    const ctx = canvas.getContext("2d");

    ctx.fillStyle = "blue";
    ctx.fillRect(x, y, w, h);

    console.log(`🟦 Drew rect: (${x}, ${y}, ${w}, ${h})`);
}

// === Optional debug overlay ===
async function loadSemanticMap() {
    try {
        const res = await fetch("ui.wpp.map.json");
        const map = await res.json();

        console.log("🧠 Semantic Map Loaded:");
        const canvas = document.getElementById("screen");
        const ctx = canvas.getContext("2d");

        for (const el of map.elements) {
            console.log(`🔎 ${el.kind} @ ${el.source ?? "N/A"} [wasm offset: ${el.wasm_offset}]`);

            if (el.kind === "box" && el.props) {
                const { x, y, width, height } = el.props;

                ctx.strokeStyle = "red";
                ctx.lineWidth = 2;
                ctx.strokeRect(x, y, width, height);
            }

            if (el.kind === "group") {
                // Optional: visualize group boundaries if desired
                // Currently omitted since group has no props (just logical nesting)
            }
        }
    } catch (err) {
        console.warn("⚠️ Failed to load semantic map:", err);
    }
}

// === Entry point ===
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
    await loadSemanticMap();
}

window.onload = runWasm;
