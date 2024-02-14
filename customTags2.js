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

class NeoFeatureCol extends HTMLElement {
  constructor() {
    super();

    const icon = this.getAttribute("icon");

    this.style.display = "block";
    this.style.textAlign = "center";
    this.style.width = "100%";
    this.style.margintop = "40px";
    this.style.marginBottom = "40px";

    const iconElement = document.createElement("i");
    iconElement.classList.add("bi");
    iconElement.classList.add(`bi-${icon}`);
    iconElement.classList.add("gradient-text");
    iconElement.style.display = "block";
    iconElement.style.fontSize = "70px";
    iconElement.style.marginBottom = "15px";

    this.prepend(iconElement);
  }
}
customElements.define("neo-feature-col", NeoFeatureCol);

class NeoContainer extends HTMLElement {
  constructor() {
    super();

    this.style.display = "block";
    this.style.margin = "auto";
    this.style.maxWidth = "1270px";
    this.style.color = "white";
    this.style.position = "relative";
  }
}
customElements.define("neo-container", NeoContainer);

function getOs() {
  const userAgent = window.navigator.userAgent.toLowerCase();

  if (userAgent.includes("Win")) return "windows";
  if (userAgent.includes("Mac")) return "apple";
  if (userAgent.includes("X11")) return "linux";
  if (userAgent.includes("Linux")) return "linux";

  return "Unknown OS";
}

class NeoDownload extends HTMLElement {
  constructor() {
    super();

    this.style.display = "block";
    this.style.width = "260px";

    this.innerHTML = `
      <button id="neo-download" style="width: 100%">
        DOWNLOAD
        <img
          src="./img/flathub.svg"
          style="margin-left: 5px; width: 25px"
        />
      </button>

      <div
        style="
          display: flex;
          align-items: center;
          opacity: 0.8;
          height: 20px;
          margin-top: 15px;
        "
      >
        <div style="font-size: 12px">Also available on:</div>
        <div
          style="
            display: flex;
            align-items: center;
            margin-left: auto;
            gap: 5px;
          "
        >
          <i class="bi bi-windows"></i>
          <i class="bi bi-apple"></i>
          <i class="bi bi-ubuntu"></i>
        </div>
      </div>
    `;
  }
}
customElements.define("neo-download", NeoDownload);
