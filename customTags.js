class NeoFloater extends HTMLElement {}
customElements.define("neo-floater", NeoFloater);

class NeoButtonGroup extends HTMLElement {
  constructor() {
    super();

    this.style.display = "flex";

    const gap = this.getAttribute("gap");

    let children = [...this.children];
    children.forEach((child, id) => {
      child.style.width = "100%";
      if (id != children.length - 1) {
        child.style.marginRight = gap != null ? gap : "5px";
      }
    });
  }
}
customElements.define("neo-button-group", NeoButtonGroup);

class NeoButton extends HTMLElement {
  constructor() {
    super();

    const href = this.getAttribute("href");
    const target = this.getAttribute("target");

    if (href != null) {
      this.addEventListener("click", () => window.open(href, target));
    }
  }
}
customElements.define("neo-button", NeoButton);

class NeoPage extends HTMLElement {}
customElements.define("neo-page", NeoPage);

class NeoPaginator extends HTMLElement {
  constructor() {
    super();
    new Pageable(this);
  }
}
customElements.define("neo-paginator", NeoPaginator);
