// Vitest setup for SolidJS testing
import { cleanup } from 'solid-js/web';
import { afterEach, vi } from 'vitest';

// Mock WebGL2
const mockGL = {
  getParameter: vi.fn(),
  getExtension: vi.fn(),
  createShader: vi.fn(),
  shaderSource: vi.fn(),
  compileShader: vi.fn(),
  getShaderParameter: vi.fn(() => true),
  getShaderInfoLog: vi.fn(() => ''),
  createProgram: vi.fn(),
  attachShader: vi.fn(),
  linkProgram: vi.fn(),
  getProgramParameter: vi.fn(() => true),
  getProgramInfoLog: vi.fn(() => ''),
  createVertexArray: vi.fn(),
  bindVertexArray: vi.fn(),
  createBuffer: vi.fn(),
  bindBuffer: vi.fn(),
  bufferData: vi.fn(),
  getAttribLocation: vi.fn(() => 0),
  enableVertexAttribArray: vi.fn(),
  vertexAttribPointer: vi.fn(),
  createTexture: vi.fn(),
  bindTexture: vi.fn(),
  texParameteri: vi.fn(),
  texImage2D: vi.fn(),
  createFramebuffer: vi.fn(),
  bindFramebuffer: vi.fn(),
  framebufferTexture2D: vi.fn(),
  viewport: vi.fn(),
  clear: vi.fn(),
  useProgram: vi.fn(),
  drawArrays: vi.fn(),
  blitFramebuffer: vi.fn(),
  activeTexture: vi.fn(),
  uniform1i: vi.fn(),
  uniform1f: vi.fn(),
  deleteProgram: vi.fn(),
  deleteShader: vi.fn(),
  deleteVertexArray: vi.fn(),
  deleteBuffer: vi.fn(),
  deleteTexture: vi.fn(),
  deleteFramebuffer: vi.fn(),
  // Constants
  VERTEX_SHADER: 0x8B31,
  FRAGMENT_SHADER: 0x8B30,
  COMPILE_STATUS: 0x8B81,
  LINK_STATUS: 0x8B82,
  TEXTURE_2D: 0x0DE1,
  TEXTURE_MIN_FILTER: 0x2801,
  NEAREST: 0x2600,
  RGBA: 0x1908,
  UNSIGNED_BYTE: 0x1401,
  FRAMEBUFFER: 0x8D40,
  COLOR_ATTACHMENT0: 0x8CE0,
  COLOR_BUFFER_BIT: 0x00004000,
  ARRAY_BUFFER: 0x8892,
  STATIC_DRAW: 0x88E4,
  FLOAT: 0x1406,
  TEXTURE0: 0x84C0,
  TEXTURE1: 0x84C1,
  TEXTURE2: 0x84C2,
  RGBA16F: 0x881A,
  R8: 0x8229,
  RED: 0x1903,
  TRIANGLE_STRIP: 0x0005,
};

HTMLCanvasElement.prototype.getContext = vi.fn((type: string) => {
  if (type === 'webgl2' || type === 'webgl') {
    return mockGL;
  }
  return null;
});

// Mock fetch for font loading
global.fetch = vi.fn(() => Promise.resolve({
  ok: true,
  text: () => Promise.resolve('00020:0000000000000000\n0002E:0000000000001818\n'),
}));

// Mock URL.createObjectURL
global.URL.createObjectURL = vi.fn(() => 'blob:mock');
global.URL.revokeObjectURL = vi.fn();

// Mock document.createElement for canvas
const originalCreateElement = document.createElement.bind(document);
document.createElement = vi.fn((tag: string) => {
  if (tag === 'canvas') {
    const canvas = originalCreateElement('canvas');
    canvas.getContext = vi.fn(() => mockGL);
    canvas.width = 800;
    canvas.height = 600;
    canvas.getBoundingClientRect = vi.fn(() => ({
      width: 800,
      height: 600,
      top: 0,
      left: 0,
      bottom: 600,
      right: 800,
    }));
    return canvas;
  }
  return originalCreateElement(tag);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Suppress console.error in tests unless explicitly needed
const originalError = console.error;
console.error = (...args) => {
  if (args[0]?.includes?.('Warning:')) return;
  originalError.apply(console, args);
};