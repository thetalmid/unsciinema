import { createSignal, createEffect, onCleanup, onMount, createMemo } from 'solid-js';

interface WebGLState {
  gl: WebGL2RenderingContext | null;
  program: WebGLProgram | null;
  vao: WebGLVertexArrayObject | null;
  buffer: WebGLBuffer | null;
  textures: {
    source: WebGLTexture | null;
    luma: WebGLTexture | null;
    atlas: WebGLTexture | null;
    output: WebGLTexture | null;
  };
  framebuffer: WebGLFramebuffer | null;
  uniformLocations: Map<string, WebGLUniformLocation | null>;
}

const initialState: WebGLState = {
  gl: null,
  program: null,
  vao: null,
  buffer: null,
  textures: { source: null, luma: null, atlas: null, output: null },
  framebuffer: null,
  uniformLocations: new Map(),
};

export function useWebGL() {
  const [canvas, setCanvas] = createSignal<HTMLCanvasElement | null>(null);
  const [state, setState] = createSignal<WebGLState>(initialState);
  const [error, setError] = createSignal<string | null>(null);

  const canvasRef = (el: HTMLCanvasElement) => {
    if (el) setCanvas(el);
  };

  // Initialize WebGL once when canvas is first available - use onMount to run only once
  onMount(() => {
    const el = canvas();
    if (!el) return;

    const gl = el.getContext('webgl2', {
      alpha: false,
      antialias: false,
      preserveDrawingBuffer: true,
    });

    if (!gl) {
      setError('WebGL2 not supported');
      return;
    }

    // Compile shaders
    const vsSource = `#version 300 es
      in vec2 aPosition;
      out vec2 vUv;
      uniform vec2 uResolution;
      uniform vec2 uImageSize;
      void main() {
        // Calculate aspect-correct scale in vertex shader
        float canvasRatio = uResolution.x / uResolution.y;
        float imageRatio = uImageSize.x / uImageSize.y;
        
        vec2 scale = vec2(1.0);
        if (imageRatio > canvasRatio) {
          // Image is wider - fit width
          scale.y = canvasRatio / imageRatio;
        } else {
          // Image is taller - fit height
          scale.x = imageRatio / canvasRatio;
        }
        
        vec2 pos = aPosition * scale;
        // UV from original position (not scaled) to cover full texture
        vUv = aPosition * 0.5 + 0.5;
        // Flip Y in UV for texture (WebGL origin bottom-left, texture top-left)
        vUv.y = 1.0 - vUv.y;
        gl_Position = vec4(pos, 0.0, 1.0);
      }
    `;

    const fsSource = `#version 300 es
      precision highp float;
      in vec2 vUv;
      out vec4 fragColor;
      
      uniform sampler2D uSource;
      
      void main() {
        fragColor = texture(uSource, vUv);
      }
    `;

    const compileShader = (type: number, source: string) => {
      const shader = gl.createShader(type)!;
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        throw new Error(gl.getShaderInfoLog(shader)!);
      }
      return shader;
    };

    const vs = compileShader(gl.VERTEX_SHADER, vsSource);
    const fs = compileShader(gl.FRAGMENT_SHADER, fsSource);
    const program = gl.createProgram()!;
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error(gl.getProgramInfoLog(program)!);
    }

    // Create VAO for full-screen quad
    const vao = gl.createVertexArray()!;
    const buffer = gl.createBuffer()!;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    const quad = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);
    const posLoc = gl.getAttribLocation(program, 'aPosition');
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);

    // Create textures
    const createTexture = () => {
      const tex = gl.createTexture()!;
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.bindTexture(gl.TEXTURE_2D, null);
      return tex;
    };

    const sourceTex = createTexture();

    // Uniform locations
    const uniformLocations = new Map<string, WebGLUniformLocation | null>();
    uniformLocations.set('uSource', gl.getUniformLocation(program, 'uSource'));
    uniformLocations.set('uResolution', gl.getUniformLocation(program, 'uResolution'));
    uniformLocations.set('uImageSize', gl.getUniformLocation(program, 'uImageSize'));

    setState({ 
      gl, 
      program, 
      vao,
      buffer,
      textures: { source: sourceTex, luma: null, atlas: null, output: null },
      framebuffer: null,
      uniformLocations,
    });

    // Cleanup on unmount only
    onCleanup(() => {
      if (gl.isContextLost()) return;
      gl.deleteProgram(program);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
      gl.deleteVertexArray(vao);
      gl.deleteBuffer(buffer);
      gl.deleteTexture(sourceTex);
    });
  });

  const resizeCanvas = () => {
    const { gl } = state();
    const el = canvas();
    if (!gl || !el) return;
    
    const dpr = window.devicePixelRatio || 1;
    const rect = el.getBoundingClientRect();
    const newWidth = Math.floor(rect.width * dpr);
    const newHeight = Math.floor(rect.height * dpr);
    
    if (el.width !== newWidth || el.height !== newHeight) {
      el.width = newWidth;
      el.height = newHeight;
      gl.viewport(0, 0, newWidth, newHeight);
    }
  };

  const uploadAtlas = (atlasData: Uint8Array, atlasWidth: number, atlasHeight: number) => {
    const { gl, textures } = state();
    if (!gl || !textures.atlas) return;
    gl.bindTexture(gl.TEXTURE_2D, textures.atlas);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, atlasWidth, atlasHeight, 0, gl.RGBA, gl.UNSIGNED_BYTE, atlasData);
    gl.bindTexture(gl.TEXTURE_2D, null);
  };

  const renderFrame = (grid: any, glyphWidth: number = 8, glyphHeight: number = 8) => {
    const { gl, program, vao, textures, uniformLocations } = state();
    const el = canvas();
    if (!gl || !program || !vao || !el) return;

    // Upload source image (pre-rendered ASCII art from WASM)
    // These are METHODS on the WASM object, not properties
    const rgba = grid.rgba ? grid.rgba() : null;
    const cols = grid.cols ? grid.cols() : 0;
    const rows = grid.rows ? grid.rows() : 0;
    
    console.log('renderFrame: rgba=', rgba ? rgba.length : null, 'cols=', cols, 'rows=', rows, 'glyph=', glyphWidth, 'x', glyphHeight);
    
    if (rgba && cols && rows) {
      const w = cols * glyphWidth;
      const h = rows * glyphHeight;
      
      gl.bindTexture(gl.TEXTURE_2D, textures.source);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, rgba);
    } else {
      console.warn('renderFrame: missing grid data', { rgba: !!rgba, cols, rows });
      return;
    }

    console.log('Canvas size:', el.width, 'x', el.height, 'Image size:', cols * glyphWidth, 'x', rows * glyphHeight);
    
    // Render directly to canvas
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, el.width, el.height);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(program);
    gl.bindVertexArray(vao);

    // Set uniforms - use actual image size (cols * glyphWidth, rows * glyphHeight)
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, textures.source);
    gl.uniform1i(uniformLocations.get('uSource')!, 0);
    gl.uniform2f(uniformLocations.get('uResolution')!, el.width, el.height);
    gl.uniform2f(uniformLocations.get('uImageSize')!, cols * glyphWidth, rows * glyphHeight);

    // Draw
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  };

  const glMemo = createMemo(() => state().gl);

  return { canvasRef, renderFrame, resizeCanvas, gl: glMemo, error, uploadAtlas };
}