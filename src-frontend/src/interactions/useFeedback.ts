import { useCallback } from 'react';
import { useDisclosure } from '../disclosure/ProgressiveDisclosure';

interface FeedbackOptions {
  successPoints?: number;
  errorPoints?: number;
  successMessage?: string;
  errorMessage?: string;
  showNotification?: boolean;
}

/**
 * A hook for providing interactive feedback when users complete actions
 * 
 * @param options - Options for customizing the feedback
 * @returns Functions for triggering success and error feedback
 */
export const useFeedback = (options: FeedbackOptions = {}) => {
  const {
    successPoints = 5,
    errorPoints = 0,
    successMessage = 'Success!',
    errorMessage = 'Error occurred',
    showNotification = true,
  } = options;
  
  const { addPoints } = useDisclosure();
  
  // Show a temporary notification
  const showFeedbackNotification = useCallback((message: string, isSuccess: boolean) => {
    if (!showNotification) return;
    
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `feedback-notification ${isSuccess ? 'success' : 'error'}`;
    notification.textContent = message;
    
    // Add to document
    document.body.appendChild(notification);
    
    // Remove after animation completes
    setTimeout(() => {
      notification.classList.add('fade-out');
      setTimeout(() => {
        if (document.body.contains(notification)) {
          document.body.removeChild(notification);
        }
      }, 300);
    }, 2000);
  }, [showNotification]);
  
  // Trigger success feedback
  const triggerSuccess = useCallback((customMessage?: string) => {
    // Add points for successful action
    if (successPoints > 0) {
      addPoints(successPoints);
    }
    
    // Show notification
    showFeedbackNotification(customMessage || successMessage, true);
    
    // Play success sound if available
    if (typeof window !== 'undefined' && window.Audio) {
      try {
        const audio = new Audio('/sounds/success.mp3');
        audio.volume = 0.5;
        audio.play().catch(() => {
          // Ignore errors - browser might block autoplay
        });
      } catch (e) {
        // Ignore errors - audio might not be supported
      }
    }
  }, [addPoints, successMessage, successPoints, showFeedbackNotification]);
  
  // Trigger error feedback
  const triggerError = useCallback((customMessage?: string) => {
    // Add/remove points for failed action
    if (errorPoints !== 0) {
      addPoints(errorPoints);
    }
    
    // Show notification
    showFeedbackNotification(customMessage || errorMessage, false);
    
    // Play error sound if available
    if (typeof window !== 'undefined' && window.Audio) {
      try {
        const audio = new Audio('/sounds/error.mp3');
        audio.volume = 0.5;
        audio.play().catch(() => {
          // Ignore errors - browser might block autoplay
        });
      } catch (e) {
        // Ignore errors - audio might not be supported
      }
    }
    
    // Add a shake animation to the body to provide haptic-like feedback
    document.body.classList.add('shake');
    setTimeout(() => {
      document.body.classList.remove('shake');
    }, 500);
  }, [addPoints, errorMessage, errorPoints, showFeedbackNotification]);
  
  return {
    triggerSuccess,
    triggerError,
  };
};

export default useFeedback;
