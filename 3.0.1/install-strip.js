(() => {
  const prompt = "Install the latest release from https://github.com/kleverhq/wavepeek/releases. Run 'wavepeek skill' to get the skill.";

  function initialize() {
    let install = document.querySelector(".playground__install");
    if (!install) {
      install = document.createElement("section");
      install.className = "playground__install";
      install.setAttribute("aria-label", "Install wavepeek");
      install.innerHTML = `
        <a href="https://github.com/kleverhq/wavepeek/releases" target="_blank" rel="noopener noreferrer">Installation instructions on GitHub Releases ↗</a>
        <span>or copy-paste this to your agent:</span>
        <div class="playground__install-prompt">
          <p id="agent-prompt" data-copy="${prompt}">Install the latest release from https://github.com/kleverhq/wavepeek/releases. Run <code>'wavepeek skill'</code> to get the skill.</p>
          <span id="copy-status" class="playground__copy-status" aria-live="polite"></span>
          <button id="copy-agent-prompt" type="button">Copy</button>
        </div>`;
      document.querySelector(".md-main").prepend(install);
    }

    const button = install.querySelector("#copy-agent-prompt");
    const status = install.querySelector("#copy-status");
    const promptElement = install.querySelector("#agent-prompt");
    const showStatus = (message) => {
      status.textContent = message;
      button.textContent = message;
      setTimeout(() => {
        if (status.textContent === message) {
          status.textContent = "";
          button.textContent = "Copy";
        }
      }, 2000);
    };

    button.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(promptElement.dataset.copy);
        showStatus("Copied");
      } catch {
        const range = document.createRange();
        range.selectNodeContents(promptElement);
        window.getSelection().removeAllRanges();
        window.getSelection().addRange(range);
        showStatus("Press Ctrl+C");
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize, { once: true });
  } else {
    initialize();
  }
})();
