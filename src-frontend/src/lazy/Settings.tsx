import React, { useState, useEffect } from 'react';
import { Input, Button } from '../components/ui/index';
import { useTheme } from '../theme/ThemeContext';
import { useAnimation } from '../animation';
import { useKeyboard } from '../keyboard';
import { useAccessibility } from '../accessibility';
import { useTour, TourButton } from '../tours';
import { useHelp, HelpButton, HelpTrigger } from '../help';
import { 
  useDisclosure, 
  ProgressiveFeature, 
  LevelProgressIndicator, 
  AdvancedFeaturesToggle, 
  FeatureBadge, 
  LockedFeatureMessage 
} from '../disclosure';
import { 
  useMicroInteraction, 
  usePressEffect, 
  useRippleEffect, 
  useFeedback 
} from '../interactions';
import './Settings.css';

// Demo tour
const demoTour = {
  id: 'settings-tour',
  name: 'Settings Tour',
  steps: [
    {
      target: '.settings-heading',
      title: 'Settings Panel',
      content: 'This is the settings panel where you can configure the application.',
      placement: 'bottom'
    },
    {
      target: '.theme-settings',
      title: 'Theme Settings',
      content: 'Here you can customize the appearance of the application.',
      placement: 'right'
    },
    {
      target: '.accessibility-settings',
      title: 'Accessibility Settings',
      content: 'These settings help make the application more accessible to everyone.',
      placement: 'left'
    },
    {
      target: '.animations-settings',
      title: 'Animation Settings',
      content: 'Control how animations and transitions behave throughout the app.',
      placement: 'right'
    },
    {
      target: '.user-level-settings',
      title: 'User Level',
      content: 'As you use the app, you\'ll earn points and unlock new features.',
      placement: 'top'
    }
  ]
};

// Demo help topics
const demoHelpTopics = [
  {
    id: 'theme-help',
    title: 'Changing Themes',
    content: `
      <h2>How to Change Themes</h2>
      <p>You can select between light, dark, or system theme. The system theme will automatically match your device settings.</p>
      <p>To change the theme:</p>
      <ol>
        <li>Go to Settings</li>
        <li>Find the "Theme" section</li>
        <li>Select your preferred theme option</li>
        <li>Changes are applied immediately</li>
      </ol>
    `,
    category: 'Appearance',
    keywords: ['theme', 'dark mode', 'light mode', 'appearance']
  },
  {
    id: 'accessibility-help',
    title: 'Accessibility Features',
    content: `
      <h2>Accessibility Options</h2>
      <p>The application includes several accessibility features to make it usable for everyone:</p>
      <ul>
        <li><strong>High Contrast:</strong> Increases contrast for better readability</li>
        <li><strong>Large Text:</strong> Makes all text larger throughout the application</li>
        <li><strong>Reduced Motion:</strong> Minimizes animations and transitions</li>
        <li><strong>Screen Reader Support:</strong> Enhances compatibility with screen readers</li>
        <li><strong>Focus Indicators:</strong> Shows clear visual indicators when navigating with keyboard</li>
      </ul>
      <p>You can access these settings through the Accessibility button or in the Settings panel.</p>
    `,
    category: 'Accessibility',
    keywords: ['accessibility', 'a11y', 'screen reader', 'high contrast', 'large text'],
    related: ['keyboard-help']
  },
  {
    id: 'keyboard-help',
    title: 'Keyboard Navigation',
    content: `
      <h2>Keyboard Shortcuts</h2>
      <p>You can navigate the application efficiently using keyboard shortcuts:</p>
      <ul>
        <li><strong>Ctrl+K:</strong> Open command palette</li>
        <li><strong>Alt+A:</strong> Open accessibility panel</li>
        <li><strong>?:</strong> Show keyboard shortcuts</li>
        <li><strong>Esc:</strong> Close current panel or dialog</li>
      </ul>
      <p>Press ? at any time to see all available keyboard shortcuts.</p>
    `,
    category: 'Navigation',
    keywords: ['keyboard', 'shortcuts', 'navigation', 'hotkeys'],
    related: ['accessibility-help']
  }
];

const Settings: React.FC = () => {
  // Demo state
  const [apiKey, setApiKey] = useState('');
  const [selectedModel, setSelectedModel] = useState('claude-3-opus-20240229');
  const [feedbackMessage, setFeedbackMessage] = useState('');
  
  // Custom hooks for enhanced UI
  const { theme, setTheme } = useTheme();
  const { animationsEnabled, animationSpeed, toggleAnimations, setAnimationSpeed } = useAnimation();
  const { registerAction } = useKeyboard();
  const { settings: a11ySettings, updateSettings: updateA11ySettings } = useAccessibility();
  const { registerTour } = useTour();
  const { registerTopic } = useHelp();
  const { userLevel, addPoints } = useDisclosure();
  
  // Interactions
  const { trigger: triggerPulse, animationClass: pulseClass } = useMicroInteraction('pulse');
  const { props: pressProps } = usePressEffect();
  const { props: rippleProps, RippleEffect } = useRippleEffect();
  const { triggerSuccess, triggerError } = useFeedback();
  
  // Register keyboard shortcuts for the settings page
  useEffect(() => {
    const unregister = registerAction({
      key: 's',
      ctrlKey: true,
      description: 'Save settings',
      handler: () => handleSaveSettings(),
      scope: 'settings'
    });
    
    return unregister;
  }, [registerAction]);
  
  // Register demo tour
  useEffect(() => {
    registerTour(demoTour);
  }, [registerTour]);
  
  // Register help topics
  useEffect(() => {
    demoHelpTopics.forEach(topic => {
      registerTopic(topic);
    });
  }, [registerTopic]);
  
  // Demo save function that awards points
  const handleSaveSettings = () => {
    if (apiKey.length < 8) {
      setFeedbackMessage('API key must be at least 8 characters');
      triggerError('Invalid API key format');
      return;
    }
    
    // Success feedback
    setFeedbackMessage('Settings saved successfully!');
    triggerSuccess();
    
    // Award points for saving settings
    addPoints(15);
    
    // Trigger a pulse animation on the save button
    triggerPulse();
    
    // In a real app, we would send this to the backend
    console.log('Saving settings:', { apiKey, selectedModel });
  };
  
  return (
    <div className="settings-container">
      <div className="settings-content">
        <div className="settings-header">
          <h1 className="settings-heading">Settings</h1>
          <div className="header-actions">
            <HelpButton />
            <TourButton tourId="settings-tour" />
          </div>
        </div>
        
        {/* User level section with progressive disclosure */}
        <div className="settings-section user-level-settings">
          <h2>Your Profile Level</h2>
          <LevelProgressIndicator />
          <AdvancedFeaturesToggle />
        </div>
        
        {/* Basic settings section */}
        <div className="settings-section">
          <h2 className="settings-heading">General Settings</h2>
          
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">API Key</label>
              <Input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your Anthropic API key"
                fullWidth
              />
              <p className="form-hint">
                Your API key is stored securely and never shared.
                <HelpTrigger topicId="api-key-help" />
              </p>
            </div>
            
            <div className="form-group">
              <label className="form-label">Default Model</label>
              <select 
                className="select-input"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
              >
                <option value="claude-3-opus-20240229">Claude 3 Opus</option>
                <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
                <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
              </select>
              <p className="form-hint">
                The model that will be used by default for new conversations.
              </p>
            </div>
          </div>
        </div>
        
        {/* Theme settings */}
        <div className="settings-section theme-settings">
          <h2 className="settings-heading">Theme Settings</h2>
          
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Theme Mode</label>
              <select 
                className="select-input"
                value={theme}
                onChange={(e) => setTheme(e.target.value as 'light' | 'dark' | 'system')}
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="system">System (Follow OS)</option>
              </select>
              <p className="form-hint">
                Choose how the application should appear.
              </p>
            </div>
          </div>
        </div>
        
        {/* Accessibility settings */}
        <div className="settings-section accessibility-settings">
          <h2 className="settings-heading">
            Accessibility
            <HelpTrigger topicId="accessibility-help" />
          </h2>
          
          <div className="settings-form">
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={a11ySettings.highContrast}
                  onChange={(e) => updateA11ySettings({ highContrast: e.target.checked })}
                />
                <span>High Contrast</span>
              </label>
              <p className="form-hint">
                Increases contrast for better readability
              </p>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={a11ySettings.largeText}
                  onChange={(e) => updateA11ySettings({ largeText: e.target.checked })}
                />
                <span>Large Text</span>
              </label>
              <p className="form-hint">
                Increases font size throughout the application
              </p>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={a11ySettings.reducedMotion}
                  onChange={(e) => updateA11ySettings({ reducedMotion: e.target.checked })}
                />
                <span>Reduced Motion</span>
              </label>
              <p className="form-hint">
                Minimizes animations and transitions
              </p>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={a11ySettings.screenReader}
                  onChange={(e) => updateA11ySettings({ screenReader: e.target.checked })}
                />
                <span>Screen Reader Support</span>
              </label>
              <p className="form-hint">
                Enhances compatibility with screen readers
              </p>
            </div>
          </div>
        </div>
        
        {/* Animation settings */}
        <div className="settings-section animations-settings">
          <h2 className="settings-heading">Animations</h2>
          
          <div className="settings-form">
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={animationsEnabled}
                  onChange={toggleAnimations}
                />
                <span>Enable Animations</span>
              </label>
              <p className="form-hint">
                Turn on/off all animations throughout the application
              </p>
            </div>
            
            <div className="form-group">
              <label className="form-label">Animation Speed</label>
              <select 
                className="select-input"
                value={animationSpeed}
                onChange={(e) => setAnimationSpeed(e.target.value as 'normal' | 'slow' | 'fast')}
                disabled={!animationsEnabled}
              >
                <option value="slow">Slow</option>
                <option value="normal">Normal</option>
                <option value="fast">Fast</option>
              </select>
              <p className="form-hint">
                Control how quickly animations play
              </p>
            </div>
          </div>
        </div>
        
        {/* Advanced settings (Progressive disclosure) */}
        <ProgressiveFeature level="intermediate" fallback={<LockedFeatureMessage level="intermediate" />}>
          <div className="settings-section">
            <h2 className="settings-heading">
              Advanced Settings 
              <FeatureBadge level="intermediate" />
            </h2>
            
            <div className="settings-form">
              <div className="form-group">
                <label className="form-label">Temperature</label>
                <div className="range-control">
                  <input 
                    type="range" 
                    min="0" 
                    max="1" 
                    step="0.1" 
                    defaultValue="0.7" 
                    className="range-input"
                  />
                  <span className="range-value">0.7</span>
                </div>
                <p className="form-hint">
                  Controls randomness in responses. Lower values are more deterministic.
                </p>
              </div>
              
              <div className="form-group">
                <label className="form-label">Max Tokens</label>
                <Input
                  type="number"
                  defaultValue="4000"
                  fullWidth
                />
                <p className="form-hint">
                  Maximum number of tokens for model responses.
                </p>
              </div>
            </div>
          </div>
        </ProgressiveFeature>
        
        {/* Expert settings (Progressive disclosure) */}
        <ProgressiveFeature level="expert" fallback={<LockedFeatureMessage level="expert" />}>
          <div className="settings-section">
            <h2 className="settings-heading">
              Expert Settings 
              <FeatureBadge level="expert" />
            </h2>
            
            <div className="settings-form">
              <div className="form-group">
                <label className="form-label">System Prompt</label>
                <textarea 
                  className="textarea-input"
                  rows={4}
                  placeholder="Enter a system prompt that will be used for all conversations"
                />
                <p className="form-hint">
                  Custom system prompt to control model behavior.
                </p>
              </div>
              
              <div className="form-group checkbox-group">
                <label className="checkbox-label">
                  <input type="checkbox" defaultChecked />
                  <span>Enable Advanced API Features</span>
                </label>
                <p className="form-hint">
                  Enables experimental API features
                </p>
              </div>
            </div>
          </div>
        </ProgressiveFeature>
        
        {/* Microinteractions Demo */}
        <div className="settings-section">
          <h2 className="settings-heading">Interaction Examples</h2>
          
          <div className="interaction-demos">
            <div className="demo-row">
              <Button 
                className={pulseClass}
                onClick={triggerPulse}
              >
                Pulse Effect
              </Button>
              
              <Button 
                {...pressProps}
              >
                Press Effect
              </Button>
              
              <div {...rippleProps} className="ripple-demo-button">
                <RippleEffect />
                Ripple Effect
              </div>
            </div>
            
            <div className="demo-row">
              <Button 
                variant="primary"
                onClick={() => triggerSuccess('Success!')}
              >
                Success Feedback
              </Button>
              
              <Button 
                variant="danger"
                onClick={() => triggerError('Something went wrong')}
              >
                Error Feedback
              </Button>
            </div>
          </div>
        </div>
        
        {/* Save button with ripple effect */}
        <div className="settings-actions">
          <p className="settings-feedback">{feedbackMessage}</p>
          
          <div {...rippleProps}>
            <Button 
              variant="primary"
              onClick={handleSaveSettings}
              className={pulseClass}
            >
              Save Settings (Ctrl+S)
              <RippleEffect />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;
