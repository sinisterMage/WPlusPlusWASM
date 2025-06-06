# 🦥 W++WASM – The Future of the Web?

> A chaotic, Python-style scripting language for .NET… now running directly in the browser via WebAssembly.  
> Built with vibes, powered by sloths, and definitely not Visual Basic.

---

## ⚡ What is W++?

W++ is a high-level scripting language designed for learning, hacking, and meme-driven development.  
Think: Python-like syntax, C#-powered backend, chaos-infused philosophy.

With **W++WASM**, we take it further:  
**No HTML. No CSS. No JS.**  
Just raw WebAssembly powered by W++ code.

---

## 🚀 Why W++WASM?

- ✅ Full rendering logic in WASM  
- ✅ Built-in GC (mark & sweep in progress)  
- ✅ Powered by `wasm_encoder` and Rust  
- ✅ UI driven from JSON layout files (soon W++ DSL directly!)  
- ✅ **No HTML/CSS/DOM required**

This is *not* a framework.  
This is a rebellion.

---

## 📦 Folder Structure

WPLUSPLUSVM/
├── parser/ # W++ core parser (C#)
├── wpp_gc_runtime/ # Rust-based mark & sweep GC (WIP)
├── wpp-wasm-runtime/ # WASM UI renderer
│ ├── index.html # Minimal HTML wrapper
│ ├── runtime.js # JS glue
│ └── ui.wasm # Output from Rust
├── out.box.json # Layout definition
└── ui.wpp # Future W++-defined UI

---
## 🛠️ Getting Started

### 1. Build the WASM UI
```sh
cd wpp-wasm-runtime
cargo run
```
### 2. Run the demo
Serve index.html using any local server

---

## 🧠 How It Works
You define your UI in out.box.json (for now)

The wpp-wasm-runtime compiles a Rust backend using wasm_encoder

Each box is drawn using imported JS canvas calls (drawRect)

We simulate object allocation with gc_alloc while preparing for real GC integration

Accessibility & semantics to be layered via .map files (see roadmap)

---

## 📈 Roadmap
 JSON layout rendering - done

 Basic wasm-only rendering pipeline - done

 Simulated memory allocation - done

 Full GC (mark & sweep) - in progress

 Semantic .map file generation - in progress

 W++-powered UI DSL - soon

 Accessibility tooling - soon

 Full replacement of HTML, CSS, JS (👀) - the dream

---

## 🤔 FAQ
Q: Is this practical?
A: No, it’s W++.

Q: Will this dethrone HTML?
A: Eventually™.

Q: Why?
A: Because we can.

---

## ❤️ Special Thanks
Everyone who starred the project (yes, all 3 of you)

Anyone brave enough to run a UI without the DOM

---

## 📜 License
MIT. Use it, break it, fork it, port it to your fridge.






