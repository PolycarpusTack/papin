import React from 'react';
import { useTour } from './TourProvider';

const TourList: React.FC = () => {
  const { availableTours, completedTours, startTour } = useTour();
  
  if (availableTours.length === 0) {
    return <p>No tours available.</p>;
  }
  
  return (
    <ul className="tour-list">
      {availableTours.map(tour => {
        const isCompleted = completedTours.includes(tour.id);
        
        return (
          <li 
            key={tour.id} 
            className={`tour-list-item ${isCompleted ? 'tour-list-item-completed' : ''}`}
            onClick={() => startTour(tour.id)}
          >
            <span className="tour-list-item-name">{tour.name}</span>
            <span 
              className={`tour-list-item-badge ${
                isCompleted 
                  ? 'tour-list-item-badge-completed' 
                  : 'tour-list-item-badge-new'
              }`}
            >
              {isCompleted ? 'Completed' : 'New'}
            </span>
          </li>
        );
      })}
    </ul>
  );
};

export default TourList;
