document$.subscribe(function () {
  if (!window.mermaid) {
    return;
  }

  window.mermaid.initialize({
    startOnLoad: false,
    securityLevel: "loose",
  });

  const diagrams = document.querySelectorAll(".mermaid");
  if (!diagrams.length) {
    return;
  }

  diagrams.forEach(function (diagram) {
    if (diagram.dataset.processed) {
      diagram.removeAttribute("data-processed");
    }
  });

  window.mermaid.run({
    nodes: diagrams,
  });
});
