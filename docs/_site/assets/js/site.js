/* fastcgi-client-rs docs site — progressive enhancement only.
   Every feature here is optional: with JavaScript disabled the page still
   renders its default (English) content, all code blocks stay readable and
   every link keeps working. */

(function () {
  "use strict";

  var LANG_KEY = "fcgi-docs-lang";

  var META = {
    en: {
      title: "fastcgi-client-rs — Async FastCGI client for Rust",
      description:
        "Runtime-agnostic FastCGI client for Rust, built on futures-io. Short connection or keep-alive, streaming responses, optional Tokio and http interop.",
      copied: "Copied",
      copy: "Copy",
      copiedLive: "Code copied to clipboard",
      copyFailed: "Copy failed, please select the code manually"
    },
    zh: {
      title: "fastcgi-client-rs — Rust 异步 FastCGI 客户端",
      description:
        "基于 futures-io 的运行时无关 Rust FastCGI 客户端。支持短连接与 keep-alive、流式响应，并可选接入 Tokio 与 http 生态。",
      copied: "已复制",
      copy: "复制",
      copiedLive: "代码已复制到剪贴板",
      copyFailed: "复制失败，请手动选择代码"
    }
  };

  var currentLang = "en";

  function live(message) {
    var region = document.getElementById("live-region");
    if (region) {
      region.textContent = "";
      region.textContent = message;
    }
  }

  /* ---------------------------------------------------------------- i18n */

  function detectLang() {
    var stored;
    try {
      stored = window.localStorage.getItem(LANG_KEY);
    } catch (err) {
      stored = null;
    }
    if (stored === "zh" || stored === "en") {
      return stored;
    }
    var nav = (navigator.language || navigator.userLanguage || "en").toLowerCase();
    return nav.indexOf("zh") === 0 ? "zh" : "en";
  }

  function applyLang(lang) {
    currentLang = META[lang] ? lang : "en";
    var meta = META[currentLang];

    document.documentElement.setAttribute("lang", currentLang === "zh" ? "zh-Hans" : "en");

    var nodes = document.querySelectorAll("[data-en]");
    for (var i = 0; i < nodes.length; i++) {
      var value = nodes[i].getAttribute("data-" + currentLang) || nodes[i].getAttribute("data-en");
      if (value !== null) {
        nodes[i].textContent = value;
      }
    }

    var labelled = document.querySelectorAll("[data-aria-en]");
    for (var j = 0; j < labelled.length; j++) {
      var label =
        labelled[j].getAttribute("data-aria-" + currentLang) || labelled[j].getAttribute("data-aria-en");
      if (label !== null) {
        labelled[j].setAttribute("aria-label", label);
      }
    }

    document.title = meta.title;
    var description = document.querySelector('meta[name="description"]');
    if (description) {
      description.setAttribute("content", meta.description);
    }

    var buttons = document.querySelectorAll("[data-lang-btn]");
    for (var k = 0; k < buttons.length; k++) {
      var isActive = buttons[k].getAttribute("data-lang-btn") === currentLang;
      buttons[k].setAttribute("aria-pressed", isActive ? "true" : "false");
    }

    // Copy buttons keep their idle label in sync with the active language.
    var copyLabels = document.querySelectorAll("[data-copy-label]");
    for (var m = 0; m < copyLabels.length; m++) {
      if (copyLabels[m].getAttribute("data-copied") !== "true") {
        setCopyLabel(copyLabels[m], meta.copy);
      }
    }
  }

  function setCopyLabel(button, text) {
    var label = button.querySelector(".copy-text");
    if (label) {
      label.textContent = text;
    }
  }

  function initLang() {
    applyLang(detectLang());

    var buttons = document.querySelectorAll("[data-lang-btn]");
    for (var i = 0; i < buttons.length; i++) {
      buttons[i].addEventListener("click", function (event) {
        var lang = event.currentTarget.getAttribute("data-lang-btn");
        applyLang(lang);
        try {
          window.localStorage.setItem(LANG_KEY, lang);
        } catch (err) {
          /* storage unavailable (private mode); language still applies */
        }
      });
    }
  }

  /* -------------------------------------------------------------- copy */

  function fallbackCopy(text) {
    var area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "-1000px";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    var ok = false;
    try {
      ok = document.execCommand("copy");
    } catch (err) {
      ok = false;
    }
    document.body.removeChild(area);
    return ok;
  }

  function flashCopied(button) {
    button.setAttribute("data-copied", "true");
    setCopyLabel(button, META[currentLang].copied);
    live(META[currentLang].copiedLive);
    window.setTimeout(function () {
      button.setAttribute("data-copied", "false");
      setCopyLabel(button, META[currentLang].copy);
    }, 1800);
  }

  function initCopy() {
    var buttons = document.querySelectorAll("[data-copy-target]");
    for (var i = 0; i < buttons.length; i++) {
      buttons[i].addEventListener("click", function (event) {
        var button = event.currentTarget;
        var target = document.getElementById(button.getAttribute("data-copy-target"));
        if (!target) {
          return;
        }
        var text = target.textContent.replace(/\s+$/, "");

        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(
            function () {
              flashCopied(button);
            },
            function () {
              if (fallbackCopy(text)) {
                flashCopied(button);
              } else {
                live(META[currentLang].copyFailed);
              }
            }
          );
        } else if (fallbackCopy(text)) {
          flashCopied(button);
        } else {
          live(META[currentLang].copyFailed);
        }
      });
    }
  }

  /* -------------------------------------------------------------- tabs */

  function initTabs() {
    var lists = document.querySelectorAll('[role="tablist"]');

    for (var i = 0; i < lists.length; i++) {
      (function (list) {
        var tabs = [].slice.call(list.querySelectorAll('[role="tab"]'));

        function select(index, focus) {
          for (var t = 0; t < tabs.length; t++) {
            var selected = t === index;
            tabs[t].setAttribute("aria-selected", selected ? "true" : "false");
            tabs[t].setAttribute("tabindex", selected ? "0" : "-1");
            var panel = document.getElementById(tabs[t].getAttribute("aria-controls"));
            if (panel) {
              if (selected) {
                panel.removeAttribute("hidden");
              } else {
                panel.setAttribute("hidden", "");
              }
            }
          }
          if (focus) {
            tabs[index].focus();
          }
        }

        for (var t = 0; t < tabs.length; t++) {
          (function (index) {
            tabs[index].addEventListener("click", function () {
              select(index, false);
            });
            tabs[index].addEventListener("keydown", function (event) {
              var next = -1;
              if (event.key === "ArrowRight" || event.key === "ArrowDown") {
                next = (index + 1) % tabs.length;
              } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
                next = (index - 1 + tabs.length) % tabs.length;
              } else if (event.key === "Home") {
                next = 0;
              } else if (event.key === "End") {
                next = tabs.length - 1;
              }
              if (next >= 0) {
                event.preventDefault();
                select(next, true);
              }
            });
          })(t);
        }
      })(lists[i]);
    }
  }

  /* ------------------------------------------------------------ reveal */

  function initReveal() {
    var items = document.querySelectorAll(".reveal");
    var reduced = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (!("IntersectionObserver" in window) || reduced) {
      for (var i = 0; i < items.length; i++) {
        items[i].classList.add("is-visible");
      }
      return;
    }

    var observer = new IntersectionObserver(
      function (entries) {
        for (var e = 0; e < entries.length; e++) {
          if (entries[e].isIntersecting) {
            entries[e].target.classList.add("is-visible");
            observer.unobserve(entries[e].target);
          }
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.08 }
    );

    for (var j = 0; j < items.length; j++) {
      observer.observe(items[j]);
    }
  }

  /* --------------------------------------------------------------- nav */

  function initNav() {
    var toggle = document.querySelector("[data-nav-toggle]");
    var nav = document.getElementById("primary-nav");

    if (toggle && nav) {
      toggle.addEventListener("click", function () {
        var open = nav.getAttribute("data-open") === "true";
        nav.setAttribute("data-open", open ? "false" : "true");
        toggle.setAttribute("aria-expanded", open ? "false" : "true");
      });

      var links = nav.querySelectorAll("a");
      for (var i = 0; i < links.length; i++) {
        links[i].addEventListener("click", function () {
          if (window.matchMedia("(max-width: 820px)").matches) {
            nav.setAttribute("data-open", "false");
            toggle.setAttribute("aria-expanded", "false");
          }
        });
      }
    }

    if (!("IntersectionObserver" in window) || !nav) {
      return;
    }

    var navLinks = [].slice.call(nav.querySelectorAll('a[href^="#"]'));
    var sections = navLinks
      .map(function (link) {
        return document.getElementById(link.getAttribute("href").slice(1));
      })
      .filter(Boolean);

    var spy = new IntersectionObserver(
      function (entries) {
        for (var e = 0; e < entries.length; e++) {
          if (!entries[e].isIntersecting) {
            continue;
          }
          var id = entries[e].target.id;
          for (var l = 0; l < navLinks.length; l++) {
            var active = navLinks[l].getAttribute("href") === "#" + id;
            navLinks[l].classList.toggle("is-current", active);
          }
        }
      },
      { rootMargin: "-45% 0px -50% 0px" }
    );

    for (var s = 0; s < sections.length; s++) {
      spy.observe(sections[s]);
    }
  }

  /* --------------------------------------------------------- highlight */

  function initHighlight() {
    if (typeof window.hljs === "undefined") {
      return; // CDN blocked or offline: plain monospace code is still readable.
    }
    var blocks = document.querySelectorAll("pre code[class*='language-']");
    for (var i = 0; i < blocks.length; i++) {
      try {
        window.hljs.highlightElement(blocks[i]);
      } catch (err) {
        /* leave the block unhighlighted */
      }
    }
  }

  /* --------------------------------------------------------------- run */

  function boot() {
    initLang();
    initCopy();
    initTabs();
    initNav();
    initReveal();
    initHighlight();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
