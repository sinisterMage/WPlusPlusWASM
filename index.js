const output = document.getElementById("output");

let memory;

const print = (ptr, len) => {
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  const text = new TextDecoder("utf-8").decode(bytes);
  output.textContent = `WASM says: ${text}`;
};

(async () => {
  const response = await fetch("hello.wasm");
  const buffer = await response.arrayBuffer();

  const { instance } = await WebAssembly.instantiate(buffer, {
    env: { print }
  });

  memory = instance.exports.memory;
  instance.exports.run();
})();
