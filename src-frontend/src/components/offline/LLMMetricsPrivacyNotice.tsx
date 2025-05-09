import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Typography, Paper, Box, Button, Stack, Checkbox, FormControlLabel, 
  Dialog, DialogTitle, DialogContent, DialogActions, Alert, Divider,
  IconButton, List, ListItem, ListItemText, ListItemIcon
} from '@mui/material';
import InfoIcon from '@mui/icons-material/Info';
import SecurityIcon from '@mui/icons-material/Security';
import PrivacyTipIcon from '@mui/icons-material/PrivacyTip';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import DoDisturbIcon from '@mui/icons-material/DoDisturb';
import AutoGraphIcon from '@mui/icons-material/AutoGraph';

interface LLMMetricsConfig {
  enabled: boolean;
  collect_performance_metrics: boolean;
  collect_usage_metrics: boolean;
  collect_error_metrics: boolean;
  anonymization_level: string;
  performance_sampling_rate: number;
  track_provider_changes: boolean;
  track_model_events: boolean;
  privacy_notice_version: string;
  privacy_notice_accepted: boolean;
}

interface PrivacyNoticeProps {
  onAccept?: () => void;
  onDecline?: () => void;
}

const CURRENT_PRIVACY_NOTICE_VERSION = "1.0.0";

const LLMMetricsPrivacyNotice: React.FC<PrivacyNoticeProps> = ({ 
  onAccept, 
  onDecline 
}) => {
  const [open, setOpen] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);
  const [config, setConfig] = useState<LLMMetricsConfig | null>(null);
  const [agreed, setAgreed] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Check if privacy notice has been accepted
  useEffect(() => {
    const checkPrivacyNotice = async () => {
      try {
        setLoading(true);
        const result = await invoke('get_llm_metrics_config');
        setConfig(result as LLMMetricsConfig);
        
        // If privacy notice hasn't been accepted or version is different, show dialog
        if (!(result as LLMMetricsConfig).privacy_notice_accepted || 
            (result as LLMMetricsConfig).privacy_notice_version !== CURRENT_PRIVACY_NOTICE_VERSION) {
          setOpen(true);
        }
      } catch (err) {
        console.error('Failed to check privacy notice status:', err);
        setError(`Failed to check privacy notice status: ${err}`);
      } finally {
        setLoading(false);
      }
    };
    
    checkPrivacyNotice();
  }, []);

  // Handle privacy notice acceptance
  const handleAccept = async () => {
    if (!agreed) {
      return;
    }
    
    try {
      setLoading(true);
      await invoke('accept_llm_metrics_privacy_notice', { 
        version: CURRENT_PRIVACY_NOTICE_VERSION 
      });
      
      if (config) {
        // Update local state
        setConfig({
          ...config,
          privacy_notice_accepted: true,
          privacy_notice_version: CURRENT_PRIVACY_NOTICE_VERSION,
          enabled: true
        });
      }
      
      setOpen(false);
      
      // Call parent callback if provided
      if (onAccept) {
        onAccept();
      }
    } catch (err) {
      console.error('Failed to accept privacy notice:', err);
      setError(`Failed to accept privacy notice: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // Handle privacy notice decline
  const handleDecline = () => {
    setOpen(false);
    
    // Call parent callback if provided
    if (onDecline) {
      onDecline();
    }
  };

  return (
    <>
      <Button 
        startIcon={<PrivacyTipIcon />}
        onClick={() => setOpen(true)}
        variant="outlined"
        color="primary"
        sx={{ mt: 2 }}
      >
        View LLM Metrics Privacy Notice
      </Button>
      
      <Dialog
        open={open}
        onClose={() => {}}
        fullWidth
        maxWidth="md"
        PaperProps={{
          sx: {
            borderRadius: 2,
            boxShadow: 5,
          }
        }}
      >
        <DialogTitle sx={{ 
          bgcolor: 'primary.main', 
          color: 'white',
          display: 'flex',
          alignItems: 'center',
          gap: 1
        }}>
          <PrivacyTipIcon />
          LLM Metrics Collection Privacy Notice
        </DialogTitle>
        
        <DialogContent dividers>
          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}
          
          <Box sx={{ mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              About LLM Performance Metrics Collection
            </Typography>
            
            <Typography paragraph>
              To improve the performance and user experience of local LLM providers, 
              we collect anonymous metrics about how the LLM providers and models perform. 
              This data helps us optimize the application, prioritize features, and understand 
              which models and providers work best.
            </Typography>
            
            <Alert severity="info" sx={{ mb: 2 }}>
              <Typography variant="subtitle2">
                This metrics collection is completely optional and disabled by default.
              </Typography>
            </Alert>
          </Box>
          
          <Box sx={{ mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              What We Collect
            </Typography>
            
            <List>
              <ListItem>
                <ListItemIcon>
                  <AutoGraphIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Performance Metrics" 
                  secondary="Response times, tokens per second, time to first token, and resource usage"
                />
              </ListItem>
              
              <ListItem>
                <ListItemIcon>
                  <InfoIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Usage Statistics" 
                  secondary="Which providers and models are used, successful/failed generations"
                />
              </ListItem>
              
              <ListItem>
                <ListItemIcon>
                  <DoDisturbIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Error Information" 
                  secondary="Types of errors that occur during model loading or generation"
                />
              </ListItem>
            </List>
            
            <Typography variant="subtitle2" color="text.secondary" sx={{ mt: 1 }}>
              We never collect any of your prompts, generated content, or personal information.
            </Typography>
          </Box>
          
          <Box sx={{ mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              Privacy Protection
            </Typography>
            
            <List>
              <ListItem>
                <ListItemIcon>
                  <SecurityIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Full Anonymization" 
                  secondary="All metrics are anonymized and cannot be traced back to you"
                />
              </ListItem>
              
              <ListItem>
                <ListItemIcon>
                  <CheckCircleIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Local Processing" 
                  secondary="Most metrics are processed locally and never leave your device"
                />
              </ListItem>
              
              <ListItem>
                <ListItemIcon>
                  <CheckCircleIcon color="primary" />
                </ListItemIcon>
                <ListItemText 
                  primary="Data Sampling" 
                  secondary="We only collect a small sample of metrics to minimize impact"
                />
              </ListItem>
            </List>
          </Box>
          
          <Box sx={{ mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              Your Control
            </Typography>
            
            <Typography paragraph>
              You have complete control over what is collected:
            </Typography>
            
            <Typography component="ul">
              <li>Enable or disable metrics collection at any time</li>
              <li>Choose which specific types of metrics are collected</li>
              <li>Adjust anonymization level to your comfort</li>
              <li>View all collected metrics in the dashboard</li>
            </Typography>
            
            <Typography variant="subtitle2" color="text.secondary" sx={{ mt: 1 }}>
              You can change these settings at any time in the Offline Settings.
            </Typography>
          </Box>
          
          <Divider sx={{ my: 2 }} />
          
          <FormControlLabel
            control={
              <Checkbox 
                checked={agreed}
                onChange={(e) => setAgreed(e.target.checked)}
                color="primary"
              />
            }
            label="I understand and agree to the collection of anonymous metrics data"
          />
        </DialogContent>
        
        <DialogActions sx={{ px: 3, py: 2 }}>
          <Button 
            onClick={handleDecline} 
            color="inherit"
            disabled={loading}
          >
            Decline
          </Button>
          
          <Button 
            onClick={handleAccept}
            variant="contained" 
            color="primary"
            disabled={loading || !agreed}
          >
            Accept and Enable Metrics
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};

export default LLMMetricsPrivacyNotice;