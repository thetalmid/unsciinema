import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/solid';
import FileDrop from '../components/FileDrop';

describe('FileDrop', () => {
  const defaultProps = {
    onLoad: vi.fn(),
    disabled: false,
    imageName: '',
    onClear: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders drop zone when no image loaded', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    expect(screen.getByText('Drop an image here or click to browse')).toBeInTheDocument();
    expect(screen.getByText('PNG, JPEG, GIF, WebP')).toBeInTheDocument();
  });

  it('shows filename when image loaded', () => {
    render(() => <FileDrop {...defaultProps} imageName="test.png" />);
    
    expect(screen.getByText('test.png')).toBeInTheDocument();
    expect(screen.getByText('✕')).toBeInTheDocument();
  });

  it('calls onLoad when file dropped', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    const file = new File(['test'], 'test.png', { type: 'image/png' });
    const dataTransfer = { files: [file] };
    
    fireEvent.drop(screen.getByRole('button'), { dataTransfer });
    
    expect(defaultProps.onLoad).toHaveBeenCalledWith(file);
  });

  it('calls onLoad when file selected via input', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    const file = new File(['test'], 'test.jpg', { type: 'image/jpeg' });
    const input = screen.getByLabelText('Drop an image here or click to browse').querySelector('input') as HTMLInputElement;
    
    Object.defineProperty(input, 'files', { value: [file] });
    fireEvent.change(input);
    
    expect(defaultProps.onLoad).toHaveBeenCalledWith(file);
  });

  it('ignores non-image files', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    const file = new File(['test'], 'test.txt', { type: 'text/plain' });
    const dataTransfer = { files: [file] };
    
    fireEvent.drop(screen.getByRole('button'), { dataTransfer });
    
    expect(defaultProps.onLoad).not.toHaveBeenCalled();
  });

  it('adds drag-over class on dragover', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    const dropZone = screen.getByRole('button');
    fireEvent.dragOver(dropZone);
    
    expect(dropZone).toHaveClass('drag-over');
  });

  it('removes drag-over class on dragleave', () => {
    render(() => <FileDrop {...defaultProps} />);
    
    const dropZone = screen.getByRole('button');
    fireEvent.dragOver(dropZone);
    fireEvent.dragLeave(dropZone);
    
    expect(dropZone).not.toHaveClass('drag-over');
  });

  it('calls onClear when clear button clicked', () => {
    render(() => <FileDrop {...defaultProps} imageName="test.png" />);
    
    fireEvent.click(screen.getByText('✕'));
    
    expect(defaultProps.onClear).toHaveBeenCalledTimes(1);
  });

  it('disables input when disabled', () => {
    render(() => <FileDrop {...defaultProps} disabled={true} />);
    
    const input = screen.getByLabelText('Drop an image here or click to browse').querySelector('input') as HTMLInputElement;
    expect(input).toBeDisabled();
  });
});