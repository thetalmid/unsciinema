export default function StatusBar(props: { message: string; glError: string | null }) {
  return (
    <footer class="status-bar">
      <span class="message">{props.message}</span>
      {props.glError && (
        <span class="error">WebGL Error: {props.glError}</span>
      )}
      <span class="info">
        Unscii font by Viznut • Rust/WASM + SolidJS + WebGL2
      </span>
    </footer>
  );
}