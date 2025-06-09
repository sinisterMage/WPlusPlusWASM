const memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
let heap = new Uint8Array(memory.buffer);
let nextAlloc = 1024;

const HEADER_SIZE = 8;
const roots = [];

// === GC bump allocator ===
function gc_alloc(size, type_id) {
    const total = size + HEADER_SIZE;
    const base = nextAlloc;

    if (base + total > heap.length) {
        console.warn(`⚠️ Out of memory at 0x${base.toString(16)}.`);
        return 0;
    }

    heap[base + 0] = type_id & 0xff;
    heap[base + 1] = (type_id >> 8) & 0xff;
    heap[base + 2] = (type_id >> 16) & 0xff;
    heap[base + 3] = (type_id >> 24) & 0xff;

    heap[base + 4] = 1;
    heap[base + 5] = 0;
    heap[base + 6] = 0;
    heap[base + 7] = 0;

    const ptr = base + HEADER_SIZE;
    nextAlloc += total;

    console.log(`🧠 Allocated ${total} bytes at 0x${base.toString(16)} (type=${type_id}, ptr=${ptr})`);
    return ptr;
}

function add_root(ptr) {
    roots.push(ptr);
    console.log(`🌱 Rooted ptr=0x${ptr.toString(16)}`);
}

function gc_collect() {
    if (typeof instance !== "undefined") {
        instance.exports.gc_tick();
        console.log("🧹 GC triggered from JS → WASM");
    } else {
        console.warn("⚠️ Cannot call GC, WASM instance not ready");
    }
}

// === Canvas draw hooks ===
function drawRect(x, y, w, h) {
    const canvas = document.getElementById("screen");
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "blue";
    ctx.fillRect(x, y, w, h);

    console.log(`🟦 drawRect(${x}, ${y}, ${w}, ${h})`);
}

function drawText(x, y, ptr, len) {
    const canvas = document.getElementById("screen");
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    const bytes = heap.subarray(ptr, ptr + len);
    const text = new TextDecoder("utf-8").decode(bytes);

    ctx.fillStyle = "white";
    ctx.font = "16px sans-serif";
    ctx.textBaseline = "top";

    // Center vertically using actual text metrics
    const metrics = ctx.measureText(text);
    const baselineOffset = metrics.actualBoundingBoxAscent / 2;
    ctx.textBaseline = "middle";

    ctx.fillText(text, x, y + baselineOffset);
    console.log(`🔤 drawText(${x}, ${y + baselineOffset}, "${text}")`);
}

// === Semantic Overlay ===
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

// === Entry Point ===
async function runWasm() {
    try {
        const wasmUrl = "ui.wasm?cachebust=" + Date.now();
        const response = await fetch(wasmUrl);
        const bytes = await response.arrayBuffer();

        const { instance: wasmInstance } = await WebAssembly.instantiate(bytes, {
            env: {
                memory,
                drawRect,
                drawText,
                gc_alloc,
                add_root,
                gc_collect,
                gc_tick: () => {},
            },
        });

        window.instance = wasmInstance;
        heap = new Uint8Array(memory.buffer);

        console.log("🚀 Running WASM program...");
        wasmInstance.exports.run();

        await loadSemanticMap();
    } catch (err) {
        console.error("❌ Failed to run W++ WASM:", err);
    }
}

window.onload = runWasm;
