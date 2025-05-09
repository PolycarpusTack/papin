// CallControls.tsx
//
// This component provides controls for audio/video calls in the collaboration session,
// including starting/ending calls and toggling audio/video.

import React, { useState, useEffect } from 'react';
import { useCollaboration } from '../../../hooks/useCollaboration';
import { Participant, MediaDevice } from '../context/CollaborationContext';

interface CallControlsProps {
  // Optional props can be added here
}

const CallControls: React.FC<CallControlsProps> = () => {
  const { state, startAudioCall, startVideoCall, endCall, toggleMute, toggleVideo } = useCollaboration();
  const { activeCall, mediaDevices, config } = state;
  
  const [isStartingCall, setIsStartingCall] = useState<boolean>(false);
  const [selectedAudioInput, setSelectedAudioInput] = useState<string>('');
  const [selectedAudioOutput, setSelectedAudioOutput] = useState<string>('');
  const [selectedVideoInput, setSelectedVideoInput] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  
  // Select default devices when list changes
  useEffect(() => {
    if (mediaDevices.length > 0) {
      // Select first audio input if none selected
      if (!selectedAudioInput) {
        const audioInput = mediaDevices.find(device => device.kind === 'audioinput');
        if (audioInput) {
          setSelectedAudioInput(audioInput.id);
        }
      }
      
      // Select first audio output if none selected
      if (!selectedAudioOutput) {
        const audioOutput = mediaDevices.find(device => device.kind === 'audiooutput');
        if (audioOutput) {
          setSelectedAudioOutput(audioOutput.id);
        }
      }
      
      // Select first video input if none selected
      if (!selectedVideoInput) {
        const videoInput = mediaDevices.find(device => device.kind === 'videoinput');
        if (videoInput) {
          setSelectedVideoInput(videoInput.id);
        }
      }
    }
  }, [mediaDevices, selectedAudioInput, selectedAudioOutput, selectedVideoInput]);
  
  // Handle starting an audio call
  const handleStartAudioCall = async () => {
    setIsStartingCall(true);
    setError(null);
    
    try {
      await startAudioCall();
    } catch (err) {
      setError(`Failed to start audio call: ${err}`);
    } finally {
      setIsStartingCall(false);
    }
  };
  
  // Handle starting a video call
  const handleStartVideoCall = async () => {
    setIsStartingCall(true);
    setError(null);
    
    try {
      await startVideoCall();
    } catch (err) {
      setError(`Failed to start video call: ${err}`);
    } finally {
      setIsStartingCall(false);
    }
  };
  
  // Handle ending a call
  const handleEndCall = async () => {
    try {
      await endCall();
    } catch (err) {
      setError(`Failed to end call: ${err}`);
    }
  };
  
  // Handle toggling mute
  const handleToggleMute = async () => {
    try {
      await toggleMute();
    } catch (err) {
      setError(`Failed to toggle mute: ${err}`);
    }
  };
  
  // Handle toggling video
  const handleToggleVideo = async () => {
    try {
      await toggleVideo();
    } catch (err) {
      setError(`Failed to toggle video: ${err}`);
    }
  };
  
  // Get current user's participant info
  const getCurrentParticipant = (): Participant | undefined => {
    if (!activeCall || !state.currentUser) return undefined;
    
    return activeCall.participants[state.currentUser.id];
  };
  
  // Filter devices by kind
  const getDevicesByKind = (kind: string): MediaDevice[] => {
    return mediaDevices.filter(device => device.kind === kind);
  };
  
  // Render device selection
  const renderDeviceSelection = () => {
    return (
      <div style={{ marginBottom: '20px' }}>
        <h4 style={{ margin: '0 0 10px 0' }}>Media Devices</h4>
        
        <div className="collaboration-form-group">
          <label 
            htmlFor="audioInput" 
            className="collaboration-label"
          >
            Microphone
          </label>
          <select
            id="audioInput"
            value={selectedAudioInput}
            onChange={(e) => setSelectedAudioInput(e.target.value)}
            className="collaboration-select"
          >
            {getDevicesByKind('audioinput').map(device => (
              <option key={device.id} value={device.id}>
                {device.name}
              </option>
            ))}
          </select>
        </div>
        
        <div className="collaboration-form-group">
          <label 
            htmlFor="audioOutput" 
            className="collaboration-label"
          >
            Speakers
          </label>
          <select
            id="audioOutput"
            value={selectedAudioOutput}
            onChange={(e) => setSelectedAudioOutput(e.target.value)}
            className="collaboration-select"
          >
            {getDevicesByKind('audiooutput').map(device => (
              <option key={device.id} value={device.id}>
                {device.name}
              </option>
            ))}
          </select>
        </div>
        
        <div className="collaboration-form-group">
          <label 
            htmlFor="videoInput" 
            className="collaboration-label"
          >
            Camera
          </label>
          <select
            id="videoInput"
            value={selectedVideoInput}
            onChange={(e) => setSelectedVideoInput(e.target.value)}
            className="collaboration-select"
          >
            {getDevicesByKind('videoinput').map(device => (
              <option key={device.id} value={device.id}>
                {device.name}
              </option>
            ))}
          </select>
        </div>
      </div>
    );
  };
  
  // Render call controls when not in a call
  const renderStartCallControls = () => {
    return (
      <div style={{ textAlign: 'center', marginTop: '20px' }}>
        <p>Start a call with users in this session</p>
        
        <div className="call-controls">
          <button
            onClick={handleStartAudioCall}
            disabled={isStartingCall}
            className="call-button primary"
          >
            <svg 
              width="20" 
              height="20" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              strokeWidth="2" 
              strokeLinecap="round" 
              strokeLinejoin="round"
            >
              <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"></path>
            </svg>
            <span style={{ marginLeft: '8px' }}>Audio Call</span>
          </button>
          
          <button
            onClick={handleStartVideoCall}
            disabled={isStartingCall}
            className="call-button primary"
          >
            <svg 
              width="20" 
              height="20" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              strokeWidth="2" 
              strokeLinecap="round" 
              strokeLinejoin="round"
            >
              <polygon points="23 7 16 12 23 17 23 7"></polygon>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
            </svg>
            <span style={{ marginLeft: '8px' }}>Video Call</span>
          </button>
        </div>
      </div>
    );
  };
  
  // Render call participant grid
  const renderCallParticipants = () => {
    if (!activeCall) return null;
    
    const participants = Object.values(activeCall.participants);
    
    return (
      <div style={{ marginBottom: '20px' }}>
        <h4 style={{ margin: '0 0 10px 0' }}>
          {activeCall.has_video ? 'Video Call' : 'Audio Call'} 
          <span style={{ fontSize: '14px', fontWeight: 'normal', marginLeft: '5px' }}>
            ({participants.length} {participants.length === 1 ? 'participant' : 'participants'})
          </span>
        </h4>
        
        <div className="call-participants">
          {participants.map(participant => (
            <div
              key={participant.user_id}
              className="participant-card"
            >
              {/* Video placeholder or avatar */}
              <div className="video-container">
                {activeCall.has_video && participant.video_enabled ? (
                  <div className="video-placeholder">
                    <svg 
                      width="48" 
                      height="48" 
                      viewBox="0 0 24 24" 
                      fill="none" 
                      stroke="#9E9E9E" 
                      strokeWidth="1" 
                      strokeLinecap="round" 
                      strokeLinejoin="round"
                    >
                      <rect x="2" y="2" width="20" height="20" rx="2" ry="2"></rect>
                      <path d="M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"></path>
                    </svg>
                  </div>
                ) : (
                  <div className="video-placeholder">
                    {participant.name.charAt(0).toUpperCase()}
                  </div>
                )}
                
                {/* Microphone status indicator */}
                <div className="media-indicator">
                  <svg 
                    width="16" 
                    height="16" 
                    viewBox="0 0 24 24" 
                    fill="none" 
                    stroke="white" 
                    strokeWidth="2" 
                    strokeLinecap="round" 
                    strokeLinejoin="round"
                  >
                    {participant.audio_enabled ? (
                      <>
                        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
                        <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                        <line x1="12" y1="19" x2="12" y2="23"></line>
                        <line x1="8" y1="23" x2="16" y2="23"></line>
                      </>
                    ) : (
                      <>
                        <line x1="1" y1="1" x2="23" y2="23"></line>
                        <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"></path>
                        <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .74-.16 1.44-.43 2.08"></path>
                        <line x1="12" y1="19" x2="12" y2="23"></line>
                        <line x1="8" y1="23" x2="16" y2="23"></line>
                      </>
                    )}
                  </svg>
                </div>
              </div>
              
              {/* Participant name */}
              <div className="participant-info">
                <div className="participant-name">
                  {participant.name}
                  {state.currentUser && participant.user_id === state.currentUser.id && ' (You)'}
                </div>
                
                {/* Speaking indicator */}
                {participant.is_speaking && (
                  <div className="participant-status" style={{ color: '#4CAF50' }}>
                    Speaking...
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  };
  
  // Render in-call controls
  const renderInCallControls = () => {
    if (!activeCall) return null;
    
    const currentParticipant = getCurrentParticipant();
    
    return (
      <div className="call-controls">
        {/* Mute/Unmute button */}
        <button
          onClick={handleToggleMute}
          className={`call-button ${currentParticipant?.audio_enabled ? 'secondary' : 'inactive'}`}
        >
          <svg 
            width="24" 
            height="24" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            strokeWidth="2" 
            strokeLinecap="round" 
            strokeLinejoin="round"
          >
            {currentParticipant?.audio_enabled ? (
              <>
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
                <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                <line x1="12" y1="19" x2="12" y2="23"></line>
                <line x1="8" y1="23" x2="16" y2="23"></line>
              </>
            ) : (
              <>
                <line x1="1" y1="1" x2="23" y2="23"></line>
                <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"></path>
                <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .74-.16 1.44-.43 2.08"></path>
                <line x1="12" y1="19" x2="12" y2="23"></line>
                <line x1="8" y1="23" x2="16" y2="23"></line>
              </>
            )}
          </svg>
        </button>
        
        {/* Video Toggle button (if this is a video call) */}
        {activeCall.has_video && (
          <button
            onClick={handleToggleVideo}
            className={`call-button ${currentParticipant?.video_enabled ? 'secondary' : 'inactive'}`}
          >
            <svg 
              width="24" 
              height="24" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              strokeWidth="2" 
              strokeLinecap="round" 
              strokeLinejoin="round"
            >
              {currentParticipant?.video_enabled ? (
                <>
                  <polygon points="23 7 16 12 23 17 23 7"></polygon>
                  <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
                </>
              ) : (
                <>
                  <path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10"></path>
                  <line x1="1" y1="1" x2="23" y2="23"></line>
                </>
              )}
            </svg>
          </button>
        )}
        
        {/* End Call button */}
        <button
          onClick={handleEndCall}
          className="call-button danger"
        >
          <svg 
            width="24" 
            height="24" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            strokeWidth="2" 
            strokeLinecap="round" 
            strokeLinejoin="round"
          >
            <path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91"></path>
            <line x1="23" y1="1" x2="1" y2="23"></line>
          </svg>
        </button>
      </div>
    );
  };
  
  // If audio/video features are not enabled
  if (!config.enable_av) {
    return (
      <div style={{ padding: '20px', textAlign: 'center' }}>
        <p>Audio/video features are disabled.</p>
        <p>Enable them in the collaboration settings.</p>
      </div>
    );
  }
  
  return (
    <div>
      {/* Error message */}
      {error && (
        <div className="collaboration-error">
          {error}
        </div>
      )}
      
      {/* Device selection */}
      {renderDeviceSelection()}
      
      {/* Call participants (if in a call) */}
      {activeCall ? renderCallParticipants() : renderStartCallControls()}
      
      {/* Call controls (if in a call) */}
      {activeCall && renderInCallControls()}
    </div>
  );
};

export default CallControls;
