import React from 'react';
import { useTheme, ThemeType } from './ThemeContext';
import './ThemeToggle.css';

export const ThemeToggle: React.FC = () => {
  const { theme, setTheme } = useTheme();
  
  const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTheme(e.target.value as ThemeType);
  };
  
  return (
    <div className="theme-toggle">
      <select 
        value={theme}
        onChange={handleThemeChange}
        className="theme-selector"
        aria-label="Select theme"
      >
        <option value="light">Light</option>
        <option value="dark">Dark</option>
        <option value="system">System</option>
      </select>
    </div>
  );
};

export default ThemeToggle;
