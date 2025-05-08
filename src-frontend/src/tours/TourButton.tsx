import React from 'react';
import { useTour } from './TourProvider';

interface TourButtonProps {
  tourId: string;
  className?: string;
}

const TourButton: React.FC<TourButtonProps> = ({ tourId, className = '' }) => {
  const { startTour, availableTours, completedTours } = useTour();
  
  const tour = availableTours.find(t => t.id === tourId);
  const isCompleted = completedTours.includes(tourId);
  
  if (!tour) return null;
  
  return (
    <button
      className={`tour-start-button ${className} ${isCompleted ? 'tour-start-button-completed' : ''}`}
      onClick={() => startTour(tourId)}
      aria-label={`Start ${tour.name} tour`}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="16" x2="12" y2="12" />
        <line x1="12" y1="8" x2="12.01" y2="8" />
      </svg>
      {isCompleted ? 'Replay Tour' : 'Take Tour'}
    </button>
  );
};

export default TourButton;
