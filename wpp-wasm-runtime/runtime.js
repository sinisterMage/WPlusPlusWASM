const memory = new WebAssembly.Memory({ initial: 1 });
let heap = new Uint8Array(memory.buffer);
let nextAlloc = 0;

// === GC bump allocator ===
function gc_alloc(size) {
    const addr = nextAlloc;
    nextAlloc += size;

    console.log(`🧠 Allocated ${size} bytes at 0x${addr.toString(16)} (${size} bytes)`);
    return addr;
}

// === Canvas draw hook ===
function drawRect(x, y, w, h) {
    const canvas = document.getElementById("screen");
    if (!canvas) {
        console.warn("⚠️ Canvas element #screen not found.");
        return;
    }

    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "blue";
    ctx.fillRect(x, y, w, h);

    console.log(`🟦 drewRect(${x}, ${y}, ${w}, ${h})`);
}

// === Optional Semantic Overlay ===
async function loadSemanticMap() {
    try {
        const res = await fetch("ui.wpp.map.json");
        const map = await res.json();

        console.log("📌 Semantic Map:");
        const canvas = document.getElementById("screen");
        const ctx = canvas.getContext("2d");

        for (const el of map.elements) {
            console.log(`🔎 ${el.kind} from ${el.source ?? "unknown"} (offset ${el.wasm_offset})`);

            if (el.kind === "box" && el.props) {
                const { x, y, width, height } = el.props;

                ctx.strokeStyle = "red";
                ctx.lineWidth = 2;
                ctx.strokeRect(x, y, width, height);
            }
        }
    } catch (err) {
        console.warn("⚠️ Failed to load semantic map:", err);
    }
}

// === Entry point ===
async function runWasm() {
    try {
        const response = await fetch("ui.wasm");
        const bytes = await response.arrayBuffer();

        const { instance } = await WebAssembly.instantiate(bytes, {
            env: {
                drawRect,
                gc_alloc,
                memory,
            },
        });

        console.log("🚀 Running WASM program...");
        instance.exports.run();

        await loadSemanticMap();
    } catch (err) {
        console.error("❌ Failed to run W++ WASM:", err);
    }
}

window.onload = runWasm;
