import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import ControlPanel from '../components/ControlPanel';

describe('ControlPanel', () => {
  const defaultProps = {
    params: {
      ditherAlgorithm: 0,
      ditherStrength: 1.0,
      threshold: 0.5,
      fontVariant: 0,
      cellSize: 8.0,
      colorMode: 0,
      palette: [0x000000FF, 0xFFFFFFFF, 0, 0, 0, 0],
      paletteSize: 2,
      contrast: 1.0,
      gamma: 1.0,
      targetWidth: 0,
    },
    setParams: vi.fn(),
    onConvert: vi.fn(),
    onExport: vi.fn(),
    disabled: false,
    converting: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders all tabs', () => {
    render(() => <ControlPanel {...defaultProps} />);
    
    expect(screen.getByText('Dither')).toBeInTheDocument();
    expect(screen.getByText('Font')).toBeInTheDocument();
    expect(screen.getByText('Render')).toBeInTheDocument();
    expect(screen.getByText('Color')).toBeInTheDocument();
    expect(screen.getByText('Advanced')).toBeInTheDocument();
  });

  it('shows dithering controls on Dither tab', () => {
    render(() => <ControlPanel {...defaultProps} />);
    
    expect(screen.getByLabelText('Algorithm')).toBeInTheDocument();
    expect(screen.getByLabelText('Strength: 1.00')).toBeInTheDocument();
    expect(screen.getByLabelText('Threshold: 0.50')).toBeInTheDocument();
  });

  it('shows font controls on Font tab', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Font'));
    
    expect(screen.getByLabelText('Font Variant')).toBeInTheDocument();
    expect(screen.getByLabelText('Cell Size: 8.0px')).toBeInTheDocument();
  });

  it('shows render controls on Render tab', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Render'));
    
    expect(screen.getByLabelText('Color Mode')).toBeInTheDocument();
    expect(screen.getByLabelText('Contrast: 1.00')).toBeInTheDocument();
    expect(screen.getByLabelText('Gamma: 1.00')).toBeInTheDocument();
    // Alpha Mode and CRT controls removed per audit
    expect(screen.queryByLabelText('Alpha Mode')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Scanlines')).not.toBeInTheDocument();
  });

  it('shows color controls on Color tab', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Color'));
    
    expect(screen.getByText('Preset Palettes')).toBeInTheDocument();
    // Palette Size only shows in Palette color mode
    expect(screen.queryByLabelText('Palette Size: 2')).not.toBeInTheDocument();
  });

  it('shows advanced controls on Advanced tab', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Advanced'));
    
    // Accessible name is 'Target Width: Autopx' (no space before px)
    expect(screen.getByLabelText('Target Width: Autopx')).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Target Width: Autopx' })).toBeInTheDocument();
    // CRT controls removed per audit
    expect(screen.queryByLabelText('Scanlines')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Curvature')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Bloom')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Vignette')).not.toBeInTheDocument();
  });

  it('calls onConvert when Convert button clicked', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Convert'));
    
    expect(defaultProps.onConvert).toHaveBeenCalledTimes(1);
  });

  it('calls onExport when Export button clicked', () => {
    render(() => <ControlPanel {...defaultProps} />);
    fireEvent.click(screen.getByText('Export'));
    
    expect(defaultProps.onExport).toHaveBeenCalledTimes(1);
  });

  it('disables buttons when disabled prop is true', () => {
    render(() => <ControlPanel {...defaultProps} disabled={true} />);
    
    expect(screen.getByText('Convert')).toBeDisabled();
    expect(screen.getByText('Export')).toBeDisabled();
    expect(screen.getByLabelText('Algorithm')).toBeDisabled();
  });

  it('shows converting state', () => {
    render(() => <ControlPanel {...defaultProps} converting={true} />);
    
    expect(screen.getByText('Converting...')).toBeInTheDocument();
    expect(screen.getByText('Converting...')).toBeDisabled();
  });

  it('updates params when sliders changed', () => {
    render(() => <ControlPanel {...defaultProps} />);
    
    const strengthSlider = screen.getByLabelText('Strength: 1.00');
    fireEvent.input(strengthSlider, { target: { value: '0.5' } });
    
    expect(defaultProps.setParams).toHaveBeenCalledWith('ditherStrength', 0.5);
  });

  it('updates params when select changed', () => {
    render(() => <ControlPanel {...defaultProps} />);
    
    const algorithmSelect = screen.getByLabelText('Algorithm');
    fireEvent.change(algorithmSelect, { target: { value: '1' } });
    
    expect(defaultProps.setParams).toHaveBeenCalledWith('ditherAlgorithm', 1);
  });
});