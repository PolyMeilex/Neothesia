class BackgroundGL {
  constructor(gl, time) {
    const vertices = [
      //
      -1.0,
      -1.0,
      //
      1.0,
      -1.0,
      //
      1.0,
      1.0,
      //
      -1.0,
      1.0,
      //
    ];

    const indices = [0, 1, 3, 3, 1, 2];

    const vertexBuffer = gl.createBuffer();
    {
      gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(vertices),
        gl.STATIC_DRAW
      );
      gl.bindBuffer(gl.ARRAY_BUFFER, null);
    }

    const indexBuffer = gl.createBuffer();
    {
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
      gl.bufferData(
        gl.ELEMENT_ARRAY_BUFFER,
        new Uint16Array(indices),
        gl.STATIC_DRAW
      );
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
    }

    const vertShader = gl.createShader(gl.VERTEX_SHADER);
    {
      const vertCode = `
    attribute vec2 pos;
      
    varying vec2 st;
    
    void main(void) {
        gl_Position = vec4(pos, 0.0, 1.0);
        st = pos.xy * 0.5 + 0.5;
    }
    `;
      gl.shaderSource(vertShader, vertCode);
      gl.compileShader(vertShader);
    }

    const fragShader = gl.createShader(gl.FRAGMENT_SHADER);
    {
      const fragCode = `
    precision mediump float;

    uniform float u_time;
    varying vec2 st;
      
    mat2 rotZ(float angle) {
        float ca = cos(angle);
        float sa = sin(angle);
        return mat2(ca, -sa, sa, ca);
    }
      
    vec3 hsv2rgb(vec3 c) {
        vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
        vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
        return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
    }

    void note_render(vec2 uv, float pos, inout vec3 color) {
        float mod_x = mod(uv.x, 0.1 * 2.5 * 2.0);

        vec3 col = vec3(160.0 / 255.0, 81.0 / 255.0, 238.0 / 255.0);

        if (pos == 0.5) {
            col = vec3(113.0 / 255.0, 48.0 / 255.0, 178.0 / 255.0);
        }

        if (uv.y > 0.0 && uv.y < 0.5) {
            color = mix(color, col,
            smoothstep(-0.002, 0., 127. / 5800. - abs(mod_x - pos)));
        }
    }
      
    #define speed -0.5
    #define liveTime 2.6

    void main(void) {
        vec2 st = st;
        vec3 color = vec3(0.12);

        float d = 0.0;

        st *= rotZ(0.7);
        st.x *= 1.5;
        st.x = mod(st.x, 0.5);

        {
            st.y += 0.5;

            float off = 0.0;
            vec2 pos = st;

            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1, color);

            off = 1.0;
            pos = st;
            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1 * 2.0, color);

            off = 3.0;
            pos = st;
            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1 * 3.0, color);

            off = 2.0;
            pos = st;
            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1 * 4.0, color);

            off = 0.0;
            pos = st;
            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1 * 5.0, color);

            off = 4.0;
            pos = st;
            pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
            note_render(pos, 0.1 * 5.0, color);
        }


        gl_FragColor = vec4(color / 2.5, 1.0);
    }
    `;
      gl.shaderSource(fragShader, fragCode);
      gl.compileShader(fragShader);
    }

    const shaderProgram = gl.createProgram();
    {
      gl.attachShader(shaderProgram, vertShader);
      gl.attachShader(shaderProgram, fragShader);
      gl.linkProgram(shaderProgram);
      gl.useProgram(shaderProgram);
    }

    {
      gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
      var coord = gl.getAttribLocation(shaderProgram, "pos");
      gl.vertexAttribPointer(coord, 2, gl.FLOAT, false, 0, 0);
      gl.enableVertexAttribArray(coord);
    }

    const timeLocation = gl.getUniformLocation(shaderProgram, "u_time");

    this.timeLocation = timeLocation;
    this.indices = indices;
    this.gl = gl;
  }

  updateTime(time) {
    this.gl.uniform1f(this.timeLocation, time / 1000.0);
  }

  draw() {
    this.gl.drawElements(
      this.gl.TRIANGLES,
      this.indices.length,
      this.gl.UNSIGNED_SHORT,
      0
    );
  }
}

class NeoBackground extends HTMLElement {
  constructor() {
    super();

    const canvas = document.createElement("canvas");
    canvas.style.width = "inherit";
    canvas.style.height = "inherit";

    this.appendChild(canvas);

    canvas.width = document.documentElement.clientWidth;
    canvas.height = document.documentElement.clientHeight;

    const gl = canvas.getContext("webgl");

    if (gl === null) {
      return;
    }

    gl.viewport(0, 0, canvas.width, canvas.height);

    gl.clearColor(12.0 / 255.0, 12.0 / 255.0, 12.0 / 255.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    let bg = new BackgroundGL(gl, 0.0);

    function loop(time) {
      bg.updateTime(time);
      bg.draw();

      window.requestAnimationFrame(loop);
    }

    window.requestAnimationFrame(loop);
  }
}
customElements.define("neo-background", NeoBackground);
