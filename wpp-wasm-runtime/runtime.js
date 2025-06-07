const memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
let heap = new Uint8Array(memory.buffer);
let nextAlloc = 1024; // start after null zone

const HEADER_SIZE = 8; // 4 bytes type + 4 bytes mark
const roots = []; // JS-side root registry (simplified)

// === GC bump allocator ===
function gc_alloc(size, type_id) {
    const total = size + HEADER_SIZE;
    const base = nextAlloc;

    if (base + total > heap.length) {
        console.warn(`⚠️ Out of memory at 0x${base.toString(16)}. Trigger GC here if needed.`);
        return 0;
    }

    // Write type_id
    heap[base + 0] = type_id & 0xff;
    heap[base + 1] = (type_id >> 8) & 0xff;
    heap[base + 2] = (type_id >> 16) & 0xff;
    heap[base + 3] = (type_id >> 24) & 0xff;

    // Write mark = 1 (live)
    heap[base + 4] = 1;
    heap[base + 5] = 0;
    heap[base + 6] = 0;
    heap[base + 7] = 0;

    const ptr = base + HEADER_SIZE;
    nextAlloc += total;

    console.log(`🧠 Allocated ${total} bytes at 0x${base.toString(16)} (type=${type_id}, ptr=${ptr})`);
    return ptr;
}

// === GC root registration ===
function add_root(ptr) {
    roots.push(ptr);
    console.log(`🌱 Rooted ptr=0x${ptr.toString(16)}`);
}

// === GC trigger (stub for now) ===
function gc_collect() {
    console.log("🧹 GC triggered (stub, no sweep logic yet)");
    // Future: mark all roots, follow children, sweep
}

// === Canvas draw hooks ===
function drawRect(x, y, w, h) {
    const canvas = document.getElementById("screen");
    if (!canvas) {
        console.warn("⚠️ Canvas element #screen not found.");
        return;
    }

    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "blue";
    ctx.fillRect(x, y, w, h);

    console.log(`🟦 drawRect(${x}, ${y}, ${w}, ${h})`);
}

function drawText(x, y, ptr, len) {
    const canvas = document.getElementById("screen");
    if (!canvas) {
        console.warn("⚠️ Canvas element #screen not found.");
        return;
    }

    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "white";
    ctx.font = "16px sans-serif";

    const bytes = heap.subarray(ptr, ptr + len);
    const text = new TextDecoder("utf-8").decode(bytes);

    ctx.fillText(text, x, y);
    console.log(`🔤 drawText(${x}, ${y}, "${text}")`);
}

// === Optional Semantic Overlay ===
async function loadSemanticMap() {
    try {
        const res = await fetch("ui.wpp.map.json?cachebust=" + Date.now());
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

            if (el.kind === "text" && el.props) {
                const { x, y, value } = el.props;
                ctx.strokeStyle = "green";
                ctx.strokeText(`[${value}]`, x, y - 10);
            }
        }
    } catch (err) {
        console.warn("⚠️ Failed to load semantic map:", err);
    }
}

// === Entry point ===
async function runWasm() {
    try {
        const wasmUrl = "ui.wasm?cachebust=" + Date.now();
        const response = await fetch(wasmUrl);
        const bytes = await response.arrayBuffer();

        const { instance } = await WebAssembly.instantiate(bytes, {
            env: {
                memory,
                drawRect,
                drawText,
                gc_alloc,
                add_root,
                gc_collect,
            },
        });

        heap = new Uint8Array(memory.buffer);

        console.log("🚀 Running WASM program...");
        instance.exports.run();

        await loadSemanticMap();
    } catch (err) {
        console.error("❌ Failed to run W++ WASM:", err);
    }
}

window.onload = runWasm;
