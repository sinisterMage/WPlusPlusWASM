// === Shared globals ===
let instance;
let heap;

// === GC memory ===
const memory = new WebAssembly.Memory({ initial: 1 }); // 64KB
const HEADER_SIZE = 8;
let nextAlloc = 1024;
const roots = [];

function gc_alloc(size, type_id) {
      console.log("⚙️ JS GC_ALLOC CALLED with", { size, type_id });
    const total = size + HEADER_SIZE;
    const base = nextAlloc;

    if (base + total > memory.buffer.byteLength) {
        console.warn(`⚠️ Out of memory at 0x${base.toString(16)}.`);
        return 0;
    }

    // Write header
    heap[base + 0] = type_id;
    heap[base + 1] = (type_id >> 8) & 0xff;
    heap[base + 2] = (type_id >> 16) & 0xff;
    heap[base + 3] = (type_id >> 24) & 0xff;

    heap[base + 4] = 1; // mark bit
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

function mark_used(ptr) {
    console.log(`🧷 mark_used(${ptr})`);
}

function gc_tick() {
    if (instance?.exports.gc_tick) {
        instance.exports.gc_tick();
        console.log("🧹 GC tick triggered");
    } else {
        console.warn("⚠️ GC tick failed — instance not ready");
    }
}

// === Canvas drawing ===
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

    if (ptr + len > heap.length || ptr < 0) {
        console.error(`❌ drawText: memory out of bounds (ptr=${ptr}, len=${len})`);
        return;
    }

    const rawBytes = heap.subarray(ptr, ptr + len);
    const decoded = new TextDecoder("utf-8").decode(rawBytes);

    ctx.fillStyle = "black"; // or even "lime", "cyan", etc.
    ctx.font = "16px sans-serif";
    ctx.textBaseline = "middle";

    const metrics = ctx.measureText(decoded);
    const baselineOffset = metrics.actualBoundingBoxAscent / 2;
    ctx.fillText(decoded, x, y + baselineOffset);
console.log(`🧠 Memory ptr=${ptr}, len=${len}`);
console.log("Heap dump:", [...heap.slice(ptr - 8, ptr + len + 8)]);

    console.log("🧠 drawText called with:");
    console.log("   ↪️ x:", x, "y:", y);
    console.log("   📦 Raw bytes:", [...rawBytes]);
    console.log("   🔤 Decoded string:", decoded);
    console.log("✅ drawText finished");
}

// === Entry Point ===
async function runWasm() {
    try {
        heap = new Uint8Array(memory.buffer);

        const wasmUrl = "ui.wasm?cachebust=" + Date.now();
        const response = await fetch(wasmUrl);
        const bytes = await response.arrayBuffer();

        const result = await WebAssembly.instantiate(bytes, {
            env: {
                memory,
                gc_alloc,
                add_root,
                mark_used,
                gc_tick,
                drawRect,
                drawText,
            },
        });

        instance = result.instance;
        heap = new Uint8Array(memory.buffer); // ensure heap points to final buffer

        console.log("🚀 Running WASM program...");
        instance.exports.run();
    } catch (err) {
        console.error("❌ Failed to run W++ WASM:", err);
    }
    console.log([...heap.slice(1032, 1032 + 12)]);

}

window.onload = runWasm;

