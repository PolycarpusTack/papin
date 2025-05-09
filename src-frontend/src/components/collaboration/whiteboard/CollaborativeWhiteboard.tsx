// CollaborativeWhiteboard.tsx
//
// This component provides a shared whiteboard that multiple users can draw on simultaneously.
// It synchronizes drawing operations between all connected users.

import React, { useRef, useEffect, useState, useCallback } from 'react';
import { useCollaboration } from '../../../hooks/useCollaboration';
import { throttle } from '../../../utils/throttle';
import { invoke } from '@tauri-apps/api/tauri';

// Types for whiteboard operations
interface Point {
  x: number;
  y: number;
}

interface DrawOperation {
  type: 'pencil' | 'line' | 'rectangle' | 'circle' | 'text' | 'eraser';
  points: Point[];
  color: string;
  size: number;
  text?: string;
  userId: string;
  timestamp: number;
}

interface WhiteboardState {
  operations: DrawOperation[];
  version: number;
}

interface CollaborativeWhiteboardProps {
  width?: number;
  height?: number;
  sessionId: string;
}

// Custom hook for synchronized whiteboard operations
const useSynchronizedWhiteboard = (sessionId: string) => {
  const { state } = useCollaboration();
  const [whiteboardState, setWhiteboardState] = useState<WhiteboardState>({
    operations: [],
    version: 0,
  });
  const [isDrawing, setIsDrawing] = useState(false);
  const [currentOperation, setCurrentOperation] = useState<DrawOperation | null>(null);
  const [tool, setTool] = useState<'pencil' | 'line' | 'rectangle' | 'circle' | 'text' | 'eraser'>('pencil');
  const [color, setColor] = useState('#000000');
  const [size, setSize] = useState(2);
  
  // Send an operation to other users
  const sendOperation = useCallback(
    throttle((operation: DrawOperation) => {
      // In a real implementation, we would use a Tauri command to send this to other users
      // For now, we'll just update our local state
      invoke('send_whiteboard_operation', { 
        sessionId, 
        operation: JSON.stringify(operation) 
      }).catch(error => {
        console.error('Failed to send whiteboard operation:', error);
      });
      
      // Optimistically update our local state
      setWhiteboardState(prev => ({
        operations: [...prev.operations, operation],
        version: prev.version + 1,
      }));
    }, 50), // throttle to 50ms
    [sessionId]
  );
  
  // Start a new drawing operation
  const startOperation = useCallback((x: number, y: number) => {
    if (!state.currentUser) return;
    
    const newOperation: DrawOperation = {
      type: tool,
      points: [{ x, y }],
      color,
      size,
      userId: state.currentUser.id,
      timestamp: Date.now(),
    };
    
    setCurrentOperation(newOperation);
    setIsDrawing(true);
  }, [tool, color, size, state.currentUser]);
  
  // Continue a drawing operation
  const continueOperation = useCallback((x: number, y: number) => {
    if (!isDrawing || !currentOperation) return;
    
    const updatedOperation = {
      ...currentOperation,
      points: [...currentOperation.points, { x, y }],
    };
    
    setCurrentOperation(updatedOperation);
  }, [isDrawing, currentOperation]);
  
  // End a drawing operation
  const endOperation = useCallback(() => {
    if (!isDrawing || !currentOperation) return;
    
    // Send the completed operation to other users
    sendOperation(currentOperation);
    
    setIsDrawing(false);
    setCurrentOperation(null);
  }, [isDrawing, currentOperation, sendOperation]);
  
  // Receive operations from other users
  useEffect(() => {
    // In a real implementation, we would listen for whiteboard operations from other users
    // For now, we'll simulate this with a mock implementation
    
    const handleIncomingOperation = (event: CustomEvent) => {
      const operation = event.detail as DrawOperation;
      
      // Don't process our own operations (we already added them)
      if (state.currentUser && operation.userId === state.currentUser.id) {
        return;
      }
      
      // Add the operation to our state
      setWhiteboardState(prev => ({
        operations: [...prev.operations, operation],
        version: prev.version + 1,
      }));
    };
    
    // Create a custom event type for whiteboard operations
    window.addEventListener('whiteboard-operation' as any, handleIncomingOperation as EventListener);
    
    return () => {
      window.removeEventListener('whiteboard-operation' as any, handleIncomingOperation as EventListener);
    };
  }, [state.currentUser]);
  
  return {
    whiteboardState,
    isDrawing,
    currentOperation,
    tool,
    color,
    size,
    setTool,
    setColor,
    setSize,
    startOperation,
    continueOperation,
    endOperation,
  };
};

const CollaborativeWhiteboard: React.FC<CollaborativeWhiteboardProps> = ({
  width = 800,
  height = 600,
  sessionId,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const {
    whiteboardState,
    isDrawing,
    currentOperation,
    tool,
    color,
    size,
    setTool,
    setColor,
    setSize,
    startOperation,
    continueOperation,
    endOperation,
  } = useSynchronizedWhiteboard(sessionId);
  
  // Convert from screen coordinates to canvas coordinates
  const getCanvasCoordinates = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    
    const rect = canvas.getBoundingClientRect();
    return {
      x: ((e.clientX - rect.left) / rect.width) * canvas.width,
      y: ((e.clientY - rect.top) / rect.height) * canvas.height,
    };
  }, []);
  
  // Mouse event handlers
  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const { x, y } = getCanvasCoordinates(e);
    startOperation(x, y);
  }, [getCanvasCoordinates, startOperation]);
  
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const { x, y } = getCanvasCoordinates(e);
    continueOperation(x, y);
  }, [getCanvasCoordinates, continueOperation]);
  
  const handleMouseUp = useCallback(() => {
    endOperation();
  }, [endOperation]);
  
  const handleMouseLeave = useCallback(() => {
    if (isDrawing) {
      endOperation();
    }
  }, [isDrawing, endOperation]);
  
  // Render all operations to the canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Clear the canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Draw all completed operations
    whiteboardState.operations.forEach(op => {
      drawOperation(ctx, op);
    });
    
    // Draw the current operation (if any)
    if (currentOperation) {
      drawOperation(ctx, currentOperation);
    }
  }, [whiteboardState, currentOperation]);
  
  // Draw a single operation to the canvas
  const drawOperation = (ctx: CanvasRenderingContext2D, op: DrawOperation) => {
    const { type, points, color, size } = op;
    
    ctx.strokeStyle = color;
    ctx.lineWidth = size;
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';
    
    switch (type) {
      case 'pencil':
        if (points.length < 2) return;
        
        ctx.beginPath();
        ctx.moveTo(points[0].x, points[0].y);
        
        for (let i = 1; i < points.length; i++) {
          ctx.lineTo(points[i].x, points[i].y);
        }
        
        ctx.stroke();
        break;
        
      case 'line':
        if (points.length < 2) return;
        
        ctx.beginPath();
        ctx.moveTo(points[0].x, points[0].y);
        ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);
        ctx.stroke();
        break;
        
      case 'rectangle':
        if (points.length < 2) return;
        
        const startPoint = points[0];
        const endPoint = points[points.length - 1];
        
        ctx.beginPath();
        ctx.rect(
          startPoint.x,
          startPoint.y,
          endPoint.x - startPoint.x,
          endPoint.y - startPoint.y
        );
        ctx.stroke();
        break;
        
      case 'circle':
        if (points.length < 2) return;
        
        const center = points[0];
        const edge = points[points.length - 1];
        const radius = Math.sqrt(
          Math.pow(edge.x - center.x, 2) + Math.pow(edge.y - center.y, 2)
        );
        
        ctx.beginPath();
        ctx.arc(center.x, center.y, radius, 0, 2 * Math.PI);
        ctx.stroke();
        break;
        
      case 'eraser':
        if (points.length < 2) return;
        
        // For the eraser, we draw in white with a larger size
        const originalStrokeStyle = ctx.strokeStyle;
        const originalLineWidth = ctx.lineWidth;
        
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = size * 2;
        
        ctx.beginPath();
        ctx.moveTo(points[0].x, points[0].y);
        
        for (let i = 1; i < points.length; i++) {
          ctx.lineTo(points[i].x, points[i].y);
        }
        
        ctx.stroke();
        
        // Restore original styles
        ctx.strokeStyle = originalStrokeStyle;
        ctx.lineWidth = originalLineWidth;
        break;
        
      case 'text':
        if (points.length < 1 || !op.text) return;
        
        ctx.font = `${size * 5}px Arial`;
        ctx.fillStyle = color;
        ctx.fillText(op.text, points[0].x, points[0].y);
        break;
    }
  };
  
  return (
    <div className="collaborative-whiteboard">
      <div className="whiteboard-toolbar">
        <div className="tool-group">
          <button
            className={`tool-button ${tool === 'pencil' ? 'active' : ''}`}
            onClick={() => setTool('pencil')}
            title="Pencil"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
            </svg>
          </button>
          <button
            className={`tool-button ${tool === 'line' ? 'active' : ''}`}
            onClick={() => setTool('line')}
            title="Line"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
          <button
            className={`tool-button ${tool === 'rectangle' ? 'active' : ''}`}
            onClick={() => setTool('rectangle')}
            title="Rectangle"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            </svg>
          </button>
          <button
            className={`tool-button ${tool === 'circle' ? 'active' : ''}`}
            onClick={() => setTool('circle')}
            title="Circle"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"></circle>
            </svg>
          </button>
          <button
            className={`tool-button ${tool === 'eraser' ? 'active' : ''}`}
            onClick={() => setTool('eraser')}
            title="Eraser"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"></path>
              <line x1="16" y1="8" x2="2" y2="22"></line>
            </svg>
          </button>
        </div>
        
        <div className="color-group">
          <input
            type="color"
            value={color}
            onChange={(e) => setColor(e.target.value)}
            title="Color"
          />
        </div>
        
        <div className="size-group">
          <input
            type="range"
            min="1"
            max="10"
            value={size}
            onChange={(e) => setSize(parseInt(e.target.value))}
            title="Size"
          />
        </div>
      </div>
      
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseLeave}
        className="whiteboard-canvas"
      />
    </div>
  );
};

export default CollaborativeWhiteboard;
