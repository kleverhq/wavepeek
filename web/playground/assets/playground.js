const DEMO_NAME = "scr1_axi.fst";
const DEMO_URL = new URL("./scr1_axi.fst", import.meta.url);
const WORKER_URL = new URL("./worker.js", import.meta.url);
const SURFER_URL = new URL("https://app.surfer-project.org/");
const DEMO_SURFER_URL = new URL(SURFER_URL);
DEMO_SURFER_URL.searchParams.set("load_url", DEMO_URL.href);
const HISTORY_LIMIT = 50;

const examples = {
  duration: `info --waves ${DEMO_NAME}`,
  dutScopes: `scope --waves ${DEMO_NAME} --filter ".*i_top.*" --max-depth 3 --tree`,
  timerValue: `value --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --signals i_timer.timer_val,i_timer.timer_en --at 66ns`,
  timerClock: `change --waves ${DEMO_NAME} --to 100ns --scope TOP.scr1_top_tb_axi.i_top.i_timer --signals clk --on "*" --sample-mode native --max 10`,
  tapFsm: `change --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top.i_core_top --signals i_tapc.tap_fsm_ff --on "posedge clk" --max 10 --row-mode sparse`,
  resets: `property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top.i_core_top --on "posedge clk" --eval "sys_rst_n_o == 0" --capture deassert --from 610ns --to 660ns`,
  mscratch: `property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top.i_core_top --on "posedge clk" --eval "i_pipe_top.i_pipe_csr.csr_mscratch_ff == 32'hf7ff8818" --capture match --max 1`,
  dmemReadCount: `property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on "posedge clk iff axi_rst_n" --eval "io_axi_dmem_arvalid && io_axi_dmem_arready" --capture match --summary --max unlimited`,
  dmemReadAddress: `property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on "posedge clk" --eval "io_axi_dmem_arvalid && io_axi_dmem_arready && (io_axi_dmem_araddr == 32'h87e)" --capture match`,
  dmemWrites: `extract axi --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --include "axi_dmem_aw" --map aclk=clk --from 960ns --to 962ns`,
  dmemTraffic: `extract axi --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --include "io_axi_dmem_(aw|w|b|ar|r)" --map aclk=clk --from 900ns --to 901ns`,
};

const commandHelp = {
  info: "info --help",
  scope: "scope --help",
  signal: "signal --help",
  value: "value --help",
  change: "change --help",
  property: "property --help",
  extract: "extract --help",
  help: "help",
};

const elements = Object.fromEntries(
  [
    "source-name", "source-size", "source-format", "source-status", "source-indicator",
    "use-demo", "open-local",
    "local-file", "open-surfer", "command-line", "command-error", "run", "clear",
    "transcript",
  ].map((id) => [id, document.getElementById(id)]),
);
const outputModes = [...document.querySelectorAll('input[name="output-mode"]')];

let activeSource;
let worker;
let runningId = 0;
let runningEntry;
let runningStarted = 0;
let commandHistory = [];
let historyIndex = null;
let historyDraft = "";
let sourceGeneration = 0;
let sourceLoading = true;

export function tokenize(command) {
  const tokens = [];
  let token = "";
  let quote = "";
  let escaped = false;
  let started = false;

  for (const character of command) {
    if (escaped) {
      token += character;
      escaped = false;
      started = true;
    } else if (character === "\\" && quote !== "'") {
      escaped = true;
      started = true;
    } else if (quote) {
      if (character === quote) quote = "";
      else token += character;
    } else if (character === "'" || character === '"') {
      quote = character;
      started = true;
    } else if (/\s/.test(character)) {
      if (started) {
        tokens.push(token);
        token = "";
        started = false;
      }
    } else {
      token += character;
      started = true;
    }
  }
  if (escaped) throw new Error("Command ends with an incomplete escape");
  if (quote) throw new Error(`Command has an unmatched ${quote} quote`);
  if (started) tokens.push(token);
  return tokens;
}

function quoteArgument(argument) {
  return /^[A-Za-z0-9_./,:+=@%^-]+$/.test(argument)
    ? argument
    : `"${argument.replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}

function renderCommand(tokens) {
  return tokens.map(quoteArgument).join(" ");
}

function setOption(tokens, option, value) {
  let insertion = tokens.length;
  for (let index = tokens.length - 1; index >= 1; index -= 1) {
    if (tokens[index] === option) {
      insertion = index;
      const hasValue = index + 1 < tokens.length && !tokens[index + 1].startsWith("-");
      tokens.splice(index, hasValue ? 2 : 1);
    } else if (tokens[index].startsWith(`${option}=`)) {
      insertion = index;
      tokens.splice(index, 1);
    }
  }
  if (value) tokens.splice(Math.min(insertion, tokens.length), 0, option, value);
}

function currentTokens() {
  const tokens = tokenize(elements["command-line"].value);
  if (tokens[0] === "wavepeek") tokens.shift();
  return ["wavepeek", ...tokens];
}

function resizeCommandLine() {
  const commandLine = elements["command-line"];
  commandLine.style.height = "";
  if (document.activeElement === commandLine) {
    commandLine.style.height = `${
      commandLine.scrollHeight + commandLine.offsetHeight - commandLine.clientHeight
    }px`;
  }
}

function writeTokens(tokens) {
  elements["command-line"].value = renderCommand(tokens.slice(1));
  synchronizeOutputMode();
  resizeCommandLine();
}

function synchronizeCommandSelection() {
  let selected;
  try {
    [selected] = tokenize(elements["command-line"].value);
    if (selected === "--help") selected = "help";
  } catch {
    selected = undefined;
  }
  for (const button of document.querySelectorAll("[data-example]")) {
    button.setAttribute("aria-pressed", String(button.dataset.example === selected));
  }
}

function synchronizeOutputMode() {
  synchronizeCommandSelection();
  try {
    const tokens = currentTokens();
    const value = tokens.includes("--jsonl") ? "jsonl" : tokens.includes("--json") ? "json" : "human";
    outputModes.find((input) => input.value === value).checked = true;
    elements["command-error"].textContent = "";
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function selectOutput(value) {
  try {
    const tokens = currentTokens().filter((token) => token !== "--json" && token !== "--jsonl");
    if (value !== "human") tokens.push(`--${value}`);
    writeTokens(tokens);
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function installExample(name) {
  const mode = outputModes.find((input) => input.checked).value;
  const tokens = ["wavepeek", ...tokenize(examples[name])];
  if (mode !== "human") tokens.push(`--${mode}`);
  writeTokens(tokens);
  elements["command-line"].focus();
}

function installCommandHelp(name) {
  writeTokens(["wavepeek", ...tokenize(commandHelp[name])]);
  elements["command-line"].focus();
}

function formatBytes(bytes) {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MiB`;
}

function formatDuration(milliseconds) {
  return milliseconds < 1000 ? `${Math.round(milliseconds)} ms` : `${(milliseconds / 1000).toFixed(2)} s`;
}

function setRunning(running) {
  elements.run.textContent = running ? "Stop" : "Run";
  elements.run.dataset.running = String(running);
}

function stopWorker() {
  if (worker) worker.terminate();
  worker = undefined;
  setRunning(false);
}

function setSource(name, bytes, kind) {
  let tokens;
  try {
    tokens = currentTokens();
  } catch {
    tokens = undefined;
  }
  if (runningEntry) stopCommand();
  else stopWorker();
  activeSource = { name, bytes: new Uint8Array(bytes), kind };
  sourceLoading = false;
  elements["source-name"].textContent = name;
  elements["source-size"].textContent = formatBytes(bytes.byteLength);
  elements["source-format"].textContent = name.split(".").pop().toUpperCase();
  elements["source-status"].textContent = "Ready";
  elements["source-indicator"].dataset.status = "ready";
  elements["use-demo"].setAttribute("aria-pressed", String(kind === "demo"));
  elements["open-local"].setAttribute("aria-pressed", String(kind === "local"));
  elements["use-demo"].classList.toggle("md-button--primary", kind === "demo");
  elements["open-surfer"].href = kind === "demo" ? DEMO_SURFER_URL.href : SURFER_URL.href;
  elements["open-surfer"].textContent = kind === "demo"
    ? "Open visually in Surfer ↗"
    : "Open Surfer and select this file ↗";
  if (tokens) {
    if (tokens[1] !== "help" && !tokens.includes("--help")) setOption(tokens, "--waves", name);
    writeTokens(tokens);
  }
}

function startSourceLoad(message) {
  if (runningEntry) stopCommand();
  sourceLoading = true;
  elements["source-status"].textContent = message;
  elements["source-indicator"].dataset.status = "loading";
}

async function useDemo() {
  const generation = ++sourceGeneration;
  startSourceLoad("Loading demo…");
  try {
    const response = await fetch(DEMO_URL);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const bytes = await response.arrayBuffer();
    if (generation === sourceGeneration) {
      setSource(DEMO_NAME, bytes, "demo");
      return true;
    }
  } catch (error) {
    if (generation === sourceGeneration) {
      sourceLoading = false;
      elements["source-status"].textContent = `Could not load demo: ${error.message}`;
      elements["source-indicator"].dataset.status = "error";
    }
  }
}

function ensureWorker() {
  if (worker) return worker;
  worker = new Worker(WORKER_URL, { type: "module" });
  worker.addEventListener("message", handleWorkerMessage);
  worker.addEventListener("error", ({ message }) => finishWithError(message));
  worker.postMessage({ type: "source", name: activeSource.name, bytes: activeSource.bytes });
  return worker;
}

function clearTranscript() {
  elements.transcript.replaceChildren();
  const empty = document.createElement("p");
  empty.className = "playground__empty";
  empty.textContent = "Terminal cleared.";
  elements.transcript.append(empty);
}

function startTranscriptEntry(command) {
  const empty = elements.transcript.querySelector(".playground__empty");
  if (empty) empty.remove();
  const entry = document.createElement("article");
  entry.className = "playground__entry";
  entry.dataset.status = "running";

  const header = document.createElement("header");
  const prompt = document.createElement("code");
  prompt.textContent = `$ ${command}`;
  const status = document.createElement("span");
  status.className = "playground__duration";
  status.textContent = "Running…";
  header.append(prompt, status);

  const stdout = document.createElement("pre");
  stdout.className = "playground__stdout";
  const stderr = document.createElement("pre");
  stderr.className = "playground__stderr";
  stderr.hidden = true;
  entry.append(header, stdout, stderr);
  elements.transcript.prepend(entry);

  while (elements.transcript.querySelectorAll(".playground__entry").length > HISTORY_LIMIT) {
    elements.transcript.querySelector(".playground__entry:last-of-type").remove();
  }
  elements.transcript.scrollTop = 0;
  return entry;
}

function finishTranscriptEntry(entry, result, duration) {
  entry.dataset.status = result.status === 0 ? "ok" : "error";
  entry.dataset.exitStatus = String(result.status);
  const durationBadge = entry.querySelector(".playground__duration");
  const outcome = result.status === 0 ? "Succeeded" : "Failed";
  durationBadge.textContent = formatDuration(duration);
  durationBadge.title = outcome;
  durationBadge.setAttribute("aria-label", `${outcome} in ${formatDuration(duration)}`);
  entry.querySelector(".playground__stdout").textContent = result.stdout || "";
  const stderr = entry.querySelector(".playground__stderr");
  stderr.textContent = result.stderr || "";
  stderr.hidden = !result.stderr;
  stderr.classList.toggle(
    "playground__stderr--progress",
    result.status === 0 &&
      stderr.textContent.trimEnd().split("\n").every((line) => line.startsWith("info: ")),
  );
  elements.transcript.scrollTop = 0;
}

function finishWithError(message) {
  if (!runningEntry) return;
  finishTranscriptEntry(
    runningEntry,
    { stdout: "", stderr: `fatal: browser: ${message}\n`, status: 1 },
    performance.now() - runningStarted,
  );
  runningEntry = undefined;
  stopWorker();
}

function handleWorkerMessage({ data }) {
  if (data.id !== runningId || !runningEntry) return;
  if (data.type === "result") {
    finishTranscriptEntry(runningEntry, data.result, performance.now() - runningStarted);
    runningEntry = undefined;
    setRunning(false);
  } else if (data.type === "error") {
    finishWithError(data.message);
  }
}

function runCommand(remember = true) {
  if (runningEntry) return;
  if (sourceLoading) {
    elements["command-error"].textContent = "Wait for the waveform to finish loading";
    return;
  }
  if (!activeSource) {
    elements["command-error"].textContent = "Choose a waveform first";
    return;
  }
  let argv;
  try {
    argv = currentTokens();
    elements["command-error"].textContent = "";
  } catch (error) {
    elements["command-error"].textContent = error.message;
    return;
  }

  runningId += 1;
  if (remember) {
    if (commandHistory.at(-1) !== elements["command-line"].value) {
      commandHistory.push(elements["command-line"].value);
      commandHistory = commandHistory.slice(-HISTORY_LIMIT);
    }
    historyIndex = null;
  }
  runningStarted = performance.now();
  runningEntry = startTranscriptEntry(renderCommand(argv));
  setRunning(true);
  try {
    ensureWorker().postMessage({ type: "run", id: runningId, argv });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    finishWithError(`Could not start browser worker: ${message}`);
  }
}

function stopCommand() {
  if (!worker || !runningEntry) return;
  runningId += 1;
  finishTranscriptEntry(
    runningEntry,
    { stdout: "", stderr: "Command stopped. Run again to start a fresh worker.\n", status: 1 },
    performance.now() - runningStarted,
  );
  runningEntry = undefined;
  stopWorker();
}

function navigateHistory(direction) {
  if (!commandHistory.length) return;
  if (historyIndex === null) {
    historyDraft = elements["command-line"].value;
    historyIndex = commandHistory.length;
  }
  historyIndex = Math.max(0, Math.min(commandHistory.length, historyIndex + direction));
  elements["command-line"].value = historyIndex === commandHistory.length
    ? historyDraft
    : commandHistory[historyIndex];
  synchronizeOutputMode();
  resizeCommandLine();
}

for (const button of document.querySelectorAll("[data-example]")) {
  button.addEventListener("click", () => installCommandHelp(button.dataset.example));
}
for (const button of document.querySelectorAll("[data-suggestion]")) {
  button.addEventListener("click", () => installExample(button.dataset.suggestion));
}
for (const input of outputModes) input.addEventListener("change", () => selectOutput(input.value));
elements["command-line"].addEventListener("focus", resizeCommandLine);
elements["command-line"].addEventListener("blur", resizeCommandLine);
elements["command-line"].addEventListener("input", () => {
  synchronizeOutputMode();
  resizeCommandLine();
});
elements["command-line"].addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    runCommand();
  } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    navigateHistory(event.key === "ArrowUp" ? -1 : 1);
  }
});
elements.run.addEventListener("click", () => {
  if (runningEntry) stopCommand();
  else runCommand();
});
elements.clear.addEventListener("click", clearTranscript);
elements["use-demo"].addEventListener("click", useDemo);
elements["open-local"].addEventListener("click", () => elements["local-file"].click());
elements["local-file"].addEventListener("change", async ({ target }) => {
  const file = target.files[0];
  if (!file) return;
  const generation = ++sourceGeneration;
  if (!/\.(vcd|fst)$/i.test(file.name)) {
    sourceLoading = false;
    elements["source-status"].textContent = "Choose a .vcd or .fst file";
    elements["source-indicator"].dataset.status = "error";
    target.value = "";
    return;
  }
  startSourceLoad(`Loading ${file.name}…`);
  try {
    const bytes = await file.arrayBuffer();
    if (generation === sourceGeneration) setSource(file.name, bytes, "local");
  } catch (error) {
    if (generation === sourceGeneration) {
      sourceLoading = false;
      elements["source-status"].textContent = `Could not load ${file.name}: ${error.message}`;
      elements["source-indicator"].dataset.status = "error";
    }
  } finally {
    target.value = "";
  }
});
elements["command-line"].value = commandHelp.help;
synchronizeOutputMode();
useDemo().then((loaded) => {
  if (loaded && elements["command-line"].value === commandHelp.help) runCommand(false);
});
