import React, { InputHTMLAttributes, forwardRef } from 'react';
import './Input.css';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  helperText?: string;
  error?: boolean;
  errorText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  fullWidth?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      helperText,
      error = false,
      errorText,
      leftIcon,
      rightIcon,
      fullWidth = false,
      className = '',
      id,
      ...props
    },
    ref
  ) => {
    // Generate a unique ID if not provided
    const inputId = id || `input-${Math.random().toString(36).substring(2, 9)}`;
    
    const inputClass = `
      input-wrapper
      ${error ? 'input-error' : ''}
      ${fullWidth ? 'input-full-width' : ''}
      ${className}
    `.trim();
    
    return (
      <div className={inputClass}>
        {label && (
          <label htmlFor={inputId} className="input-label">
            {label}
          </label>
        )}
        
        <div className="input-container">
          {leftIcon && <span className="input-icon input-icon-left">{leftIcon}</span>}
          
          <input
            ref={ref}
            id={inputId}
            className={`
              input-field
              ${leftIcon ? 'input-with-left-icon' : ''}
              ${rightIcon ? 'input-with-right-icon' : ''}
            `}
            aria-invalid={error}
            {...props}
          />
          
          {rightIcon && <span className="input-icon input-icon-right">{rightIcon}</span>}
        </div>
        
        {(helperText || (error && errorText)) && (
          <div className={`input-helper-text ${error ? 'input-error-text' : ''}`}>
            {error ? errorText : helperText}
          </div>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

export default Input;
