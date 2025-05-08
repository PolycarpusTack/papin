import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import './tours.css';

// Step interface for tour
export interface TourStep {
  target: string;
  title: string;
  content: string;
  placement?: 'top' | 'right' | 'bottom' | 'left';
  disableOverlay?: boolean;
}

// Tour interface
export interface Tour {
  id: string;
  name: string;
  steps: TourStep[];
}

// Progress tracking
interface TourProgress {
  tourId: string;
  currentStep: number;
  completed: boolean;
}

// Context interface
interface TourContextType {
  activeTour: TourProgress | null;
  availableTours: Tour[];
  completedTours: string[];
  startTour: (tourId: string) => void;
  endTour: () => void;
  nextStep: () => void;
  prevStep: () => void;
  skipTour: () => void;
  registerTour: (tour: Tour) => void;
  resetTourHistory: () => void;
}

const TourContext = createContext<TourContextType>({
  activeTour: null,
  availableTours: [],
  completedTours: [],
  startTour: () => {},
  endTour: () => {},
  nextStep: () => {},
  prevStep: () => {},
  skipTour: () => {},
  registerTour: () => {},
  resetTourHistory: () => {},
});

export const useTour = () => useContext(TourContext);

interface TourProviderProps {
  children: ReactNode;
}

export const TourProvider: React.FC<TourProviderProps> = ({ children }) => {
  const [availableTours, setAvailableTours] = useState<Tour[]>([]);
  const [activeTour, setActiveTour] = useState<TourProgress | null>(null);
  const [completedTours, setCompletedTours] = useState<string[]>(() => {
    const saved = localStorage.getItem('mcp-completed-tours');
    return saved ? JSON.parse(saved) : [];
  });

  // Save completed tours to localStorage
  useEffect(() => {
    localStorage.setItem('mcp-completed-tours', JSON.stringify(completedTours));
  }, [completedTours]);

  // Register a new tour
  const registerTour = (tour: Tour) => {
    setAvailableTours(prev => {
      // Check if tour already exists
      const exists = prev.some(t => t.id === tour.id);
      if (exists) {
        return prev.map(t => t.id === tour.id ? tour : t);
      }
      return [...prev, tour];
    });
  };

  // Start a tour
  const startTour = (tourId: string) => {
    const tour = availableTours.find(t => t.id === tourId);
    if (!tour) {
      console.error(`Tour with id ${tourId} not found`);
      return;
    }

    setActiveTour({
      tourId,
      currentStep: 0,
      completed: false,
    });
  };

  // End a tour
  const endTour = () => {
    if (activeTour) {
      setCompletedTours(prev => {
        if (!prev.includes(activeTour.tourId)) {
          return [...prev, activeTour.tourId];
        }
        return prev;
      });
    }
    setActiveTour(null);
  };

  // Go to next step
  const nextStep = () => {
    if (!activeTour) return;

    const tour = availableTours.find(t => t.id === activeTour.tourId);
    if (!tour) return;

    if (activeTour.currentStep < tour.steps.length - 1) {
      setActiveTour({
        ...activeTour,
        currentStep: activeTour.currentStep + 1,
      });
    } else {
      setActiveTour({
        ...activeTour,
        completed: true,
      });
      endTour();
    }
  };

  // Go to previous step
  const prevStep = () => {
    if (!activeTour) return;

    if (activeTour.currentStep > 0) {
      setActiveTour({
        ...activeTour,
        currentStep: activeTour.currentStep - 1,
      });
    }
  };

  // Skip the tour
  const skipTour = () => {
    endTour();
  };

  // Reset tour history
  const resetTourHistory = () => {
    setCompletedTours([]);
    localStorage.removeItem('mcp-completed-tours');
  };

  // Scroll to the target element and highlight it
  useEffect(() => {
    if (!activeTour) return;

    const tour = availableTours.find(t => t.id === activeTour.tourId);
    if (!tour) return;

    const step = tour.steps[activeTour.currentStep];
    if (!step) return;

    const target = document.querySelector(step.target);
    if (!target) return;

    // Scroll to target if needed
    target.scrollIntoView({
      behavior: 'smooth',
      block: 'center',
    });

    // Add highlight class to target
    target.classList.add('tour-target');

    return () => {
      target.classList.remove('tour-target');
    };
  }, [activeTour, availableTours]);

  return (
    <TourContext.Provider
      value={{
        activeTour,
        availableTours,
        completedTours,
        startTour,
        endTour,
        nextStep,
        prevStep,
        skipTour,
        registerTour,
        resetTourHistory,
      }}
    >
      {children}
      {activeTour && <TourOverlay />}
    </TourContext.Provider>
  );
};

// Helper function to calculate best placement
const calculateBestPlacement = (targetRect: DOMRect): 'top' | 'right' | 'bottom' | 'left' => {
  const windowHeight = window.innerHeight;
  const windowWidth = window.innerWidth;
  
  // Calculate available space in each direction
  const topSpace = targetRect.top;
  const rightSpace = windowWidth - (targetRect.left + targetRect.width);
  const bottomSpace = windowHeight - (targetRect.top + targetRect.height);
  const leftSpace = targetRect.left;
  
  // Find direction with most space
  const spaces = [
    { direction: 'bottom', space: bottomSpace },
    { direction: 'left', space: leftSpace },
    { direction: 'right', space: rightSpace },
    { direction: 'top', space: topSpace },
  ];
  
  spaces.sort((a, b) => b.space - a.space);
  
  return spaces[0].direction as 'top' | 'right' | 'bottom' | 'left';
};

// Helper function to calculate tooltip position
const calculateTooltipPosition = (
  targetRect: DOMRect, 
  placement: 'top' | 'right' | 'bottom' | 'left'
): { top: number; left: number; } => {
  const OFFSET = 12; // Distance between target and tooltip
  
  switch (placement) {
    case 'top':
      return {
        top: targetRect.top - OFFSET,
        left: targetRect.left + targetRect.width / 2,
      };
    case 'right':
      return {
        top: targetRect.top + targetRect.height / 2,
        left: targetRect.right + OFFSET,
      };
    case 'bottom':
      return {
        top: targetRect.bottom + OFFSET,
        left: targetRect.left + targetRect.width / 2,
      };
    case 'left':
      return {
        top: targetRect.top + targetRect.height / 2,
        left: targetRect.left - OFFSET,
      };
  }
};

// Tour overlay component
const TourOverlay: React.FC = () => {
  const { activeTour, availableTours, nextStep, prevStep, skipTour } = useTour();

  if (!activeTour) return null;

  const tour = availableTours.find(t => t.id === activeTour.tourId);
  if (!tour) return null;

  const step = tour.steps[activeTour.currentStep];
  if (!step) return null;

  const target = document.querySelector(step.target);
  if (!target) return null;

  // Get target position for tooltip placement
  const targetRect = target.getBoundingClientRect();
  const placement = step.placement || calculateBestPlacement(targetRect);

  // Calculate tooltip position
  const tooltipPosition = calculateTooltipPosition(targetRect, placement);

  return (
    <div className="tour-overlay">
      {!step.disableOverlay && <div className="tour-overlay-background" />}
      
      <div 
        className={`tour-tooltip tour-tooltip-${placement}`}
        style={{
          top: `${tooltipPosition.top}px`,
          left: `${tooltipPosition.left}px`,
        }}
      >
        <div className="tour-tooltip-arrow" />
        
        <div className="tour-tooltip-content">
          <div className="tour-tooltip-header">
            <h3>{step.title}</h3>
            <button 
              className="tour-tooltip-close" 
              onClick={skipTour}
              aria-label="Close tour"
            >
              &times;
            </button>
          </div>
          
          <div className="tour-tooltip-body">
            {step.content}
          </div>
          
          <div className="tour-tooltip-footer">
            <div className="tour-tooltip-progress">
              {activeTour.currentStep + 1} of {tour.steps.length}
            </div>
            
            <div className="tour-tooltip-actions">
              {activeTour.currentStep > 0 && (
                <button 
                  className="tour-tooltip-button tour-tooltip-button-secondary" 
                  onClick={prevStep}
                >
                  Previous
                </button>
              )}
              
              <button 
                className="tour-tooltip-button tour-tooltip-button-primary" 
                onClick={nextStep}
              >
                {activeTour.currentStep < tour.steps.length - 1 ? 'Next' : 'Finish'}
              </button>
              
              <button
                className="tour-tooltip-button tour-tooltip-button-text"
                onClick={skipTour}
              >
                Skip
              </button>
            </div>
          </div>
        </div>
      </div>
      
      {/* Highlight the target element */}
      <div 
        className="tour-target-highlight"
        style={{
          top: `${targetRect.top}px`,
          left: `${targetRect.left}px`,
          width: `${targetRect.width}px`,
          height: `${targetRect.height}px`,
        }}
      />
    </div>
  );
};

export default TourProvider;
