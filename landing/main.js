/* Minimal interactions: mobile nav, year, copy install, scroll reveal.
   Respects prefers-reduced-motion. */

(() => {
  const yearEl = document.querySelector("[data-year]");
  if (yearEl) yearEl.textContent = String(new Date().getFullYear());

  /* Mobile nav */
  const toggle = document.querySelector("[data-nav-toggle]");
  const drawer = document.querySelector("[data-nav-drawer]");
  if (toggle && drawer) {
    const setOpen = (open) => {
      drawer.classList.toggle("open", open);
      drawer.hidden = !open;
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    };
    setOpen(false);
    toggle.addEventListener("click", () => {
      setOpen(!drawer.classList.contains("open"));
    });
    drawer.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", () => setOpen(false));
    });
  }

  /* Copy install command */
  const copyBtn = document.querySelector("[data-copy-install]");
  const cmdEl = document.querySelector("[data-install-cmd]");
  if (copyBtn && cmdEl) {
    const label = copyBtn.querySelector("[data-copy-label]");
    const iconCopy = copyBtn.querySelector(".icon-copy");
    const iconCheck = copyBtn.querySelector(".icon-check");
    let resetTimer;

    const setCopied = (copied) => {
      copyBtn.classList.toggle("is-copied", copied);
      if (label) label.textContent = copied ? "Copied" : "Copy";
      if (iconCopy) iconCopy.hidden = copied;
      if (iconCheck) iconCheck.hidden = !copied;
    };

    copyBtn.addEventListener("click", async () => {
      const text = cmdEl.textContent.trim();
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(text);
        } else {
          const ta = document.createElement("textarea");
          ta.value = text;
          ta.setAttribute("readonly", "");
          ta.style.position = "absolute";
          ta.style.left = "-9999px";
          document.body.appendChild(ta);
          ta.select();
          document.execCommand("copy");
          document.body.removeChild(ta);
        }
        setCopied(true);
        clearTimeout(resetTimer);
        resetTimer = setTimeout(() => setCopied(false), 1800);
      } catch {
        setCopied(false);
      }
    });
  }

  /* Scroll reveal */
  const reduce =
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  if (!reduce && "IntersectionObserver" in window) {
    const nodes = document.querySelectorAll(".reveal");
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("is-in");
            io.unobserve(entry.target);
          }
        });
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.12 }
    );
    nodes.forEach((n) => io.observe(n));
  } else {
    document.querySelectorAll(".reveal").forEach((n) => n.classList.add("is-in"));
  }
})();
