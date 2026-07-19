const themeButton = document.querySelector("[data-theme-toggle]");
const terminalTip = document.querySelector("[data-terminal-tip]");
const terminalTipsSource = document.querySelector("#terminal-tips");
const themeLabels = {
  light: "Dark",
  dark: "Light",
};

function currentTheme() {
  return document.documentElement.dataset.theme === "dark" ? "dark" : "light";
}

function syncThemeButton(button: Element) {
  const theme = currentTheme();

  button.setAttribute("aria-label", `Switch to ${themeLabels[theme].toLowerCase()} theme`);
}

function installThemeToggle(button: Element) {
  button.addEventListener("click", () => {
    const nextTheme = currentTheme() === "dark" ? "light" : "dark";

    document.documentElement.dataset.theme = nextTheme;
    try {
      localStorage.setItem("gitfleet-site-theme", nextTheme);
    } catch {
      // Theme changes still apply for the current page even when storage is unavailable.
    }

    syncThemeButton(button);
  });

  syncThemeButton(button);
}

function installTerminalTips(target: Element, source: Element) {
  const terminalTips = JSON.parse(source.textContent ?? "[]") as string[];

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    window.setTimeout(() => {
      target.textContent = terminalTips[0] ?? "";
    }, 450);

    return;
  }

  let tipIndex = 0;
  let charIndex = 0;
  let deleting = false;

  function typeTerminalTip() {
    const phrase = terminalTips[tipIndex] ?? "";

    target.textContent = phrase.slice(0, charIndex);

    if (deleting) {
      if (charIndex === 0) {
        deleting = false;
        tipIndex = (tipIndex + 1) % terminalTips.length;
        window.setTimeout(typeTerminalTip, 260);
        return;
      }

      charIndex -= 1;
    } else {
      if (charIndex === phrase.length) {
        deleting = true;
        window.setTimeout(typeTerminalTip, 1800);
        return;
      }

      charIndex += 1;
    }

    window.setTimeout(typeTerminalTip, deleting ? 22 : 38);
  }

  window.setTimeout(typeTerminalTip, 700);
}

if (themeButton) {
  installThemeToggle(themeButton);
}

if (terminalTip && terminalTipsSource) {
  installTerminalTips(terminalTip, terminalTipsSource);
}
