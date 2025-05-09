import React, { useState } from 'react';
import './ExtendedGuide.css';

interface ExtendedGuideProps {
  onClose: () => void;
}

const ExtendedGuide: React.FC<ExtendedGuideProps> = ({ onClose }) => {
  const [currentChapter, setCurrentChapter] = useState(0);
  const [currentSection, setCurrentSection] = useState(0);
  
  const chapters = [
    {
      title: "Chapter 1: Introduction to Papin",
      sections: [
        {
          title: "Learning Objectives",
          content: (
            <ul className="guide-objectives">
              <li>Understand what Papin is and how it enhances AI interactions</li>
              <li>Recognize the key benefits of using Papin as your MCP client</li>
              <li>Identify the core features that set Papin apart</li>
              <li>Prepare for successful installation and configuration</li>
            </ul>
          )
        },
        {
          title: "What is Papin?",
          content: (
            <div>
              <p>Papin is a sophisticated desktop application that serves as a Model Communication Protocol (MCP) client. Think of it as your personal interface for interacting with artificial intelligence models, designed to provide seamless access whether you're online or offline.</p>
              
              <div className="info-bubble">
                <strong>Did You Know?</strong> Papin is named after Denis Papin, a 17th-century physicist and inventor known for his work on steam engines. Just as Papin helped harness steam power for practical use, our application helps make AI power more accessible and practical for everyday tasks.
              </div>
              
              <p>At its core, Papin bridges the gap between sophisticated AI technology and practical, everyday use by:</p>
              
              <ul>
                <li><strong>Enabling offline capabilities</strong> through local model integration</li>
                <li><strong>Enhancing privacy</strong> with local-first processing options</li>
                <li><strong>Providing robust performance monitoring</strong> to optimize your experience</li>
                <li><strong>Ensuring reliable synchronization</strong> across your devices</li>
              </ul>
              
              <div className="metaphor-key">
                <h4>Metaphor Key: Understanding Papin</h4>
                <p><strong>Papin as a Universal Translator:</strong> Imagine you're in a room with brilliant minds who speak different languages. Papin acts as your interpreter, helping you communicate with AI models regardless of your technical knowledge or network connectivity.</p>
                <p><strong>Papin as a Smart Workspace:</strong> Think of Papin as your dedicated office that intelligently adapts to your needs. It organizes your AI conversations, manages resources efficiently, and ensures your work is backed up and available wherever you go.</p>
              </div>
            </div>
          )
        },
        {
          title: "Core Features and Benefits",
          content: (
            <div>
              <div className="feature-grid">
                <div className="feature-card">
                  <h4>Offline Capabilities</h4>
                  <p>Work with AI models even without internet connectivity, ensuring productivity regardless of where you are.</p>
                </div>
                <div className="feature-card">
                  <h4>Local-First Architecture</h4>
                  <p>Keep your data on your device first, enhancing privacy and reducing latency for faster interactions.</p>
                </div>
                <div className="feature-card">
                  <h4>Performance Monitoring</h4>
                  <p>Gain insights into system resources, model performance, and application metrics to optimize your experience.</p>
                </div>
                <div className="feature-card">
                  <h4>Seamless Synchronization</h4>
                  <p>Maintain consistency across multiple devices with intelligent conflict resolution and efficient sync protocols.</p>
                </div>
              </div>
              
              <div className="use-case">
                <h4>Real-World Scenario: Mobile Researcher</h4>
                <p>Dr. Chen is a field researcher who often works in remote locations with limited connectivity. Before discovering Papin, she had to prepare all her research questions in advance and wait until returning to areas with internet access to get AI assistance.</p>
                <p>With Papin, Dr. Chen downloads specialized research models before her expeditions. While in the field, she can continue asking complex questions about her observations, generating hypotheses, and analyzing preliminary data—all without internet connectivity. When she returns to camp with WiFi, Papin automatically synchronizes her conversations to the cloud, making them available on her lab computer and ensuring no insights are lost.</p>
              </div>
            </div>
          )
        },
        {
          title: "Your Learning Path",
          content: (
            <div>
              <p>This comprehensive guide will take you from complete beginner to advanced user through a carefully structured learning path:</p>
              
              <div className="learning-path">
                <div className="path-stage">
                  <div className="stage-number">1</div>
                  <div className="stage-content">
                    <h4>Foundation</h4>
                    <p>Installation, basic setup, and your first AI conversation</p>
                  </div>
                </div>
                <div className="path-stage">
                  <div className="stage-number">2</div>
                  <div className="stage-content">
                    <h4>Core Features</h4>
                    <p>Mastering conversations, organization, and offline capabilities</p>
                  </div>
                </div>
                <div className="path-stage">
                  <div className="stage-number">3</div>
                  <div className="stage-content">
                    <h4>Advanced Techniques</h4>
                    <p>Performance tuning, customization, and specialized workflows</p>
                  </div>
                </div>
                <div className="path-stage">
                  <div className="stage-number">4</div>
                  <div className="stage-content">
                    <h4>Expert Integration</h4>
                    <p>API usage, programmatic control, and enterprise workflows</p>
                  </div>
                </div>
              </div>
              
              <div className="concept-checkpoint">
                <h4>Quick Check: Understanding Papin's Purpose</h4>
                <div className="question">
                  <p>What is the primary advantage of Papin's local-first architecture?</p>
                  <div className="options">
                    <label className="option">
                      <input type="radio" name="q1" value="a" />
                      <span>Reduces installation size</span>
                    </label>
                    <label className="option">
                      <input type="radio" name="q1" value="b" />
                      <span>Enhances privacy and enables offline use</span>
                    </label>
                    <label className="option">
                      <input type="radio" name="q1" value="c" />
                      <span>Eliminates the need for synchronization</span>
                    </label>
                  </div>
                  <div className="feedback feedback-correct">
                    <p>Correct! Papin's local-first approach prioritizes keeping your data on your device, enhancing privacy and enabling offline functionality.</p>
                  </div>
                </div>
              </div>
            </div>
          )
        },
        {
          title: "Practice Exercise",
          content: (
            <div className="practice-exercise">
              <h4>Envisioning Your Papin Usage</h4>
              
              <p>Consider your unique needs and workflow to prepare for an effective Papin setup:</p>
              
              <ol>
                <li>Identify 2-3 scenarios in your work or personal life where you'd benefit from AI assistance</li>
                <li>For each scenario, note whether internet connectivity is consistently available</li>
                <li>Consider what types of questions or tasks you'd typically need help with</li>
                <li>Think about your device constraints (storage space, memory, processing power)</li>
              </ol>
              
              <div className="exercise-template">
                <h5>Sample Template:</h5>
                <pre>{`Scenario 1: Writing technical documentation
Connectivity: Usually available, occasional travel
Typical tasks: Structuring content, improving clarity, generating examples
Device constraints: 16GB RAM, 256GB SSD (75GB free)

Scenario 2: Research data analysis
Connectivity: Limited (fieldwork)
Typical tasks: Data interpretation, methodology suggestions, literature connections
Device constraints: Same laptop as above`}</pre>
              </div>
              
              <p className="exercise-note">This exercise will help you make better decisions about which local models to install, how to configure offline settings, and which features to prioritize in your setup.</p>
            </div>
          )
        },
        {
          title: "Key Takeaways",
          content: (
            <div>
              <ul className="key-takeaways">
                <li>Papin is a sophisticated MCP client that enables AI interaction with enhanced offline capabilities.</li>
                <li>The local-first architecture provides benefits for privacy, performance, and accessibility.</li>
                <li>Core features include offline capabilities, performance monitoring, and seamless synchronization.</li>
                <li>Understanding your specific use cases helps optimize your Papin configuration.</li>
              </ul>
              
              <div className="growth-mindset">
                <h4>Growth Mindset Moment</h4>
                <p>Learning a new tool like Papin might seem daunting at first, especially if you're new to AI technologies. Remember that mastery comes through exploration and practice. Each feature you learn builds upon the last, creating a foundation for advanced usage. Don't worry about understanding everything immediately—this guide is structured to introduce concepts gradually, building your confidence as you progress.</p>
              </div>
              
              <div className="apply-to-life">
                <h4>Apply It To Your Life</h4>
                <p>Consider areas in your life where consistent AI assistance could enhance your productivity or creativity. Perhaps you're a student needing research help in places with unreliable internet, a professional who travels frequently, or someone who values privacy in AI interactions. Identifying these scenarios now will help you recognize the value of specific Papin features as we explore them.</p>
              </div>
            </div>
          )
        }
      ]
    },
    {
      title: "Chapter 2: Getting Started with Papin",
      sections: [
        {
          title: "Learning Objectives",
          content: (
            <ul className="guide-objectives">
              <li>Successfully install Papin on your preferred operating system</li>
              <li>Create and configure your Papin account</li>
              <li>Navigate the Papin interface</li>
              <li>Initiate and complete your first AI conversation</li>
            </ul>
          )
        },
        {
          title: "Installation Process",
          content: (
            <div>
              <p>Installing Papin is straightforward across all major operating systems. Choose the instructions for your platform:</p>
              
              <div className="tabbed-content">
                <div className="tab-headers">
                  <div className="tab-header active">Windows</div>
                  <div className="tab-header">macOS</div>
                  <div className="tab-header">Linux</div>
                </div>
                <div className="tab-content active">
                  <ol>
                    <li>Download the installer (.msi file) from the <a href="#">official Papin website</a>.</li>
                    <li>Run the downloaded .msi file.</li>
                    <li>If prompted by User Account Control, click "Yes" to allow the installer to run.</li>
                    <li>Follow the on-screen instructions in the installation wizard.</li>
                    <li>Choose your installation location or accept the default.</li>
                    <li>Select whether to create a desktop shortcut and Start menu entry.</li>
                    <li>Click "Install" to begin the installation process.</li>
                    <li>Once complete, click "Finish" to exit the installer.</li>
                  </ol>
                  <div className="info-bubble">
                    <strong>Tip:</strong> If you're installing in an enterprise environment, you can use the silent installation option with <code>papin-installer.exe /S</code> for automated deployment.
                  </div>
                </div>
                <div className="tab-content">
                  <ol>
                    <li>Download the .dmg file from the <a href="#">official Papin website</a>.</li>
                    <li>Open the downloaded .dmg file.</li>
                    <li>In the window that appears, drag the Papin icon to the Applications folder.</li>
                    <li>Eject the disk image by dragging it to the Trash, which becomes an Eject button when a disk image is selected.</li>
                    <li>Open your Applications folder and double-click on Papin to launch it.</li>
                    <li>If you see a security warning, go to System Preferences > Security & Privacy, and click "Open Anyway."</li>
                  </ol>
                  <div className="info-bubble">
                    <strong>Note:</strong> On newer versions of macOS, you may need to right-click (or Control+click) on the app and select "Open" the first time to bypass Gatekeeper.
                  </div>
                </div>
                <div className="tab-content">
                  <ol>
                    <li>Download the appropriate package for your distribution (.deb, .rpm, or .AppImage).</li>
                    <li>For .deb packages (Ubuntu, Debian, etc.):
                      <pre>sudo dpkg -i papin_1.0.0.deb</pre>
                      <pre>sudo apt-get install -f # To resolve any dependencies</pre>
                    </li>
                    <li>For .rpm packages (Fedora, RHEL, etc.):
                      <pre>sudo rpm -i papin_1.0.0.rpm</pre>
                      <pre>sudo dnf install -f # To resolve any dependencies</pre>
                    </li>
                    <li>For .AppImage:
                      <pre>chmod +x Papin-1.0.0.AppImage</pre>
                      <pre>./Papin-1.0.0.AppImage</pre>
                    </li>
                  </ol>
                  <div className="info-bubble">
                    <strong>Tip:</strong> You can integrate AppImage files with your desktop environment using tools like AppImageLauncher.
                  </div>
                </div>
              </div>
              
              <div className="mini-knowledge-drop">
                <h4>Installation Verification</h4>
                <p>After installation, Papin automatically runs a verification process to ensure all components are properly installed. If you encounter any issues, you can manually trigger this verification by running <code>papin --verify</code> from the command line.</p>
              </div>
            </div>
          )
        },
        {
          title: "Initial Setup and Configuration",
          content: (
            <div>
              <p>When you first launch Papin, you'll be guided through a setup wizard to create your account and configure basic settings:</p>
              
              <div className="setup-steps">
                <div className="setup-step">
                  <div className="step-number">1</div>
                  <div className="step-content">
                    <h4>Welcome Screen</h4>
                    <p>Papin will present a welcome screen with options to create a new account or sign in with an existing one. Click "Create Account" to get started.</p>
                  </div>
                </div>
                <div className="setup-step">
                  <div className="step-number">2</div>
                  <div className="step-content">
                    <h4>Account Creation</h4>
                    <p>Enter your email address and create a secure password. This account will be used to synchronize your conversations and settings across devices.</p>
                    <div className="info-bubble">
                      <strong>Privacy Note:</strong> All user data is encrypted both in transit and at rest. Papin's local-first architecture means your data stays on your device by default.
                    </div>
                  </div>
                </div>
                <div className="setup-step">
                  <div className="step-number">3</div>
                  <div className="step-content">
                    <h4>Model Selection</h4>
                    <p>Choose which AI models you want to use. You can select from cloud-based models (which require internet connectivity) and local models (which run directly on your device).</p>
                    <div className="tip-box">
                      <strong>Recommendation:</strong> For a balanced experience, select at least one general-purpose local model for offline use and several specialized cloud models for when you have connectivity.
                    </div>
                  </div>
                </div>
                <div className="setup-step">
                  <div className="step-number">4</div>
                  <div className="step-content">
                    <h4>Offline Capabilities</h4>
                    <p>Configure your offline settings, including whether to download models for offline use, how much disk space to allocate, and synchronization preferences.</p>
                  </div>
                </div>
                <div className="setup-step">
                  <div className="step-number">5</div>
                  <div className="step-content">
                    <h4>Performance Settings</h4>
                    <p>Papin will automatically detect your system capabilities and suggest optimal performance settings. You can adjust these settings based on your preferences for speed vs. resource usage.</p>
                  </div>
                </div>
                <div className="setup-step">
                  <div className="step-number">6</div>
                  <div className="step-content">
                    <h4>Setup Complete</h4>
                    <p>Once configuration is complete, Papin will finish downloading any selected local models (if you opted for offline capabilities) and present you with the main interface.</p>
                  </div>
                </div>
              </div>
              
              <div className="concept-checkpoint">
                <h4>Quick Check: Initial Setup</h4>
                <div className="question">
                  <p>Which of the following is NOT part of the initial Papin setup process?</p>
                  <div className="options">
                    <label className="option">
                      <input type="radio" name="q2" value="a" />
                      <span>Account creation</span>
                    </label>
                    <label className="option">
                      <input type="radio" name="q2" value="b" />
                      <span>Model selection</span>
                    </label>
                    <label className="option">
                      <input type="radio" name="q2" value="c" />
                      <span>Creating your first conversation template</span>
                    </label>
                  </div>
                  <div className="feedback feedback-correct">
                    <p>Correct! Creating conversation templates is an advanced feature you can explore after the initial setup, not part of the onboarding process.</p>
                  </div>
                </div>
              </div>
            </div>
          )
        },
        {
          title: "Navigating the Interface",
          content: (
            <div>
              <p>The Papin interface is designed to be intuitive and focused on your conversations. Let's explore the main elements:</p>
              
              <div className="interface-overview">
                <div className="interface-section">
                  <h4>Main Layout</h4>
                  <div className="interface-image">
                    [Interface Diagram: Main Layout]
                  </div>
                  <ul>
                    <li><strong>Sidebar:</strong> Contains your conversation history, folders, and navigation options</li>
                    <li><strong>Main Content Area:</strong> Displays the current conversation or other active content</li>
                    <li><strong>Toolbar:</strong> Provides access to settings, help, and additional features</li>
                    <li><strong>Status Bar:</strong> Shows connection status, sync progress, and system metrics</li>
                  </ul>
                </div>
                
                <div className="interface-section">
                  <h4>Sidebar Navigation</h4>
                  <ul>
                    <li><strong>New Conversation:</strong> Button to start a new AI conversation</li>
                    <li><strong>Search Bar:</strong> Search across all your conversations</li>
                    <li><strong>Recent Conversations:</strong> Quick access to recently active conversations</li>
                    <li><strong>Folders:</strong> Organizational structure for your conversations</li>
                    <li><strong>Favorites:</strong> Starred or pinned conversations for easy access</li>
                  </ul>
                </div>
                
                <div className="interface-section">
                  <h4>Conversation Interface</h4>
                  <ul>
                    <li><strong>Conversation Title:</strong> Shows the name of your current conversation (editable)</li>
                    <li><strong>Message History:</strong> Scrollable area showing the conversation between you and the AI</li>
                    <li><strong>Input Area:</strong> Where you type your messages to the AI</li>
                    <li><strong>Model Information:</strong> Shows which AI model is currently active</li>
                    <li><strong>Settings:</strong> Access conversation-specific settings and model parameters</li>
                  </ul>
                </div>
              </div>
              
              <div className="keyboard-shortcuts">
                <h4>Essential Keyboard Shortcuts</h4>
                <table>
                  <thead>
                    <tr>
                      <th>Action</th>
                      <th>Windows/Linux</th>
                      <th>macOS</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>New Conversation</td>
                      <td>Ctrl+N</td>
                      <td>⌘+N</td>
                    </tr>
                    <tr>
                      <td>Find in Conversation</td>
                      <td>Ctrl+F</td>
                      <td>⌘+F</td>
                    </tr>
                    <tr>
                      <td>Search All Conversations</td>
                      <td>Ctrl+Shift+F</td>
                      <td>⌘+Shift+F</td>
                    </tr>
                    <tr>
                      <td>Settings</td>
                      <td>Ctrl+,</td>
                      <td>⌘+,</td>
                    </tr>
                    <tr>
                      <td>Help</td>
                      <td>F1</td>
                      <td>F1</td>
                    </tr>
                    <tr>
                      <td>Manual Sync</td>
                      <td>Ctrl+Shift+S</td>
                      <td>⌘+Shift+S</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              
              <div className="mini-knowledge-drop">
                <h4>Interface Customization</h4>
                <p>You can customize the Papin interface by going to Settings > Appearance. Options include dark/light theme, sidebar width, font size, and density settings for the conversation display.</p>
              </div>
            </div>
          )
        },
        {
          title: "Your First Conversation",
          content: (
            <div>
              <p>Now that you're familiar with the interface, let's start your first conversation with an AI model:</p>
              
              <ol className="first-conversation-steps">
                <li>
                  <strong>Start a new conversation</strong>
                  <p>Click the "New Conversation" button in the sidebar or use the Ctrl+N (⌘+N on Mac) keyboard shortcut.</p>
                </li>
                <li>
                  <strong>Select an AI model</strong>
                  <p>You'll be prompted to choose which AI model to use. For your first conversation, select a general-purpose model that best matches your needs.</p>
                  <div className="info-bubble">
                    <strong>Model Selection Tip:</strong> Models with a "Local" badge run directly on your device and work offline. Models with a "Cloud" badge require internet connectivity but may have more advanced capabilities.
                  </div>
                </li>
                <li>
                  <strong>Type your first message</strong>
                  <p>In the input area at the bottom of the conversation, type your question or message. Be clear and specific for the best results.</p>
                  <div className="example-message">
                    <p><strong>Example Message:</strong> "Hello! I'd like to learn more about quantum computing. Can you explain the basic principles in simple terms and provide some examples of potential applications?"</p>
                  </div>
                </li>
                <li>
                  <strong>Send your message</strong>
                  <p>Press Enter or click the Send button (arrow icon) to send your message to the AI model.</p>
                </li>
                <li>
                  <strong>Review the response</strong>
                  <p>The AI will process your message and generate a response. This may take a few moments depending on the model and your hardware.</p>
                </li>
                <li>
                  <strong>Continue the conversation</strong>
                  <p>You can follow up with additional questions, ask for clarification, or steer the conversation in new directions. The AI maintains context from previous messages.</p>
                </li>
              </ol>
              
              <div className="use-case">
                <h4>Example Conversation Flow</h4>
                <div className="conversation-example">
                  <div className="message user-message">
                    <strong>You:</strong> Hello! I'd like to learn more about quantum computing. Can you explain the basic principles in simple terms and provide some examples of potential applications?
                  </div>
                  <div className="message ai-message">
                    <strong>AI:</strong> Certainly! Let me explain quantum computing in simple terms.
                    
                    <p><strong>Basic Principles:</strong></p>
                    <p>Classical computers use bits (0s and 1s) as their basic unit of information. Quantum computers use quantum bits or "qubits" which can exist in multiple states simultaneously thanks to two quantum phenomena:</p>
                    <ul>
                      <li><strong>Superposition:</strong> Unlike a classical bit which must be either 0 OR 1, a qubit can be in a state that is effectively both 0 AND 1 at the same time.</li>
                      <li><strong>Entanglement:</strong> Qubits can be linked together so that the state of one instantly affects the state of another, regardless of distance.</li>
                    </ul>
                    
                    <p><strong>Think of it like this:</strong> If you flip a coin, it lands on either heads or tails. But a "quantum coin" could be spinning in a state that represents both heads and tails simultaneously until you measure it.</p>
                    
                    <p><strong>Potential Applications:</strong></p>
                    <ul>
                      <li><strong>Cryptography:</strong> Breaking (and creating) more complex encryption systems</li>
                      <li><strong>Drug Discovery:</strong> Modeling complex molecular interactions to develop new medications faster</li>
                      <li><strong>Optimization Problems:</strong> Solving complex logistics challenges like traffic flow or supply chain management</li>
                      <li><strong>Materials Science:</strong> Designing new materials with specific properties by modeling quantum interactions</li>
                    </ul>
                    
                    <p>Would you like me to explain any of these principles or applications in more detail?</p>
                  </div>
                  <div className="message user-message">
                    <strong>You:</strong> Thanks for the explanation! Could you tell me more about how quantum computing could impact drug discovery specifically?
                  </div>
                  <div className="message ai-message">
                    <strong>AI:</strong> [Response about quantum computing in drug discovery would appear here]
                  </div>
                </div>
              </div>
              
              <div className="practice-exercise">
                <h4>Your Turn: First Conversation Practice</h4>
                <p>Now it's your turn to start your first conversation in Papin. Choose a topic you're genuinely interested in learning about, as this will make the experience more engaging and useful. Here are some starter ideas:</p>
                <ul>
                  <li>Ask for an explanation of a complex concept in your field</li>
                  <li>Request help brainstorming ideas for a project</li>
                  <li>Ask for a summary of a recent development in your area of interest</li>
                  <li>Seek guidance on learning a new skill or topic</li>
                </ul>
                <p>After your conversation, consider:</p>
                <ul>
                  <li>How clear was the AI's response?</li>
                  <li>Did the AI maintain context when you asked follow-up questions?</li>
                  <li>How might you phrase your questions differently next time?</li>
                </ul>
              </div>
            </div>
          )
        },
        {
          title: "Key Takeaways",
          content: (
            <div>
              <ul className="key-takeaways">
                <li>Installing Papin is straightforward across Windows, macOS, and Linux platforms.</li>
                <li>The initial setup process configures your account, model selection, and offline capabilities.</li>
                <li>The Papin interface is organized around a sidebar for navigation and a main area for conversations.</li>
                <li>Starting a new conversation involves selecting a model and typing your message in the input area.</li>
                <li>The AI maintains context throughout the conversation, allowing for natural follow-up questions.</li>
              </ul>
              
              <div className="growth-mindset">
                <h4>Growth Mindset Moment</h4>
                <p>Your first conversations with AI models might not be perfect. Learning to effectively communicate with AI is a skill that develops over time. Don't be discouraged if you don't get the exact responses you're looking for at first. Experiment with different ways of phrasing your questions, providing context, and guiding the conversation. With practice, you'll develop an intuition for how to get the most out of these interactions.</p>
              </div>
              
              <div className="peer-reflection">
                <h4>Reflection Prompt</h4>
                <p>Consider sharing your experience starting with Papin with a colleague or friend. Compare notes on:</p>
                <ul>
                  <li>Which models did you choose and why?</li>
                  <li>What types of questions yielded the most useful responses?</li>
                  <li>How might you integrate Papin into your daily workflow?</li>
                </ul>
              </div>
              
              <div className="apply-to-life">
                <h4>Apply It To Your Life</h4>
                <p>Identify one specific task in your workflow that could benefit from AI assistance through Papin. It could be drafting emails, researching topics, brainstorming ideas, or analyzing data. Set up a dedicated folder for this task and experiment with using Papin to enhance your productivity in this area over the next week.</p>
              </div>
            </div>
          )
        }
      ]
    }
  ];
  
  const nextChapter = () => {
    if (currentChapter < chapters.length - 1) {
      setCurrentChapter(currentChapter + 1);
      setCurrentSection(0);
      window.scrollTo(0, 0);
    }
  };
  
  const prevChapter = () => {
    if (currentChapter > 0) {
      setCurrentChapter(currentChapter - 1);
      setCurrentSection(0);
      window.scrollTo(0, 0);
    }
  };
  
  const nextSection = () => {
    const currentChapterObj = chapters[currentChapter];
    if (currentSection < currentChapterObj.sections.length - 1) {
      setCurrentSection(currentSection + 1);
      window.scrollTo(0, 0);
    } else if (currentChapter < chapters.length - 1) {
      nextChapter();
    }
  };
  
  const prevSection = () => {
    if (currentSection > 0) {
      setCurrentSection(currentSection - 1);
      window.scrollTo(0, 0);
    } else if (currentChapter > 0) {
      setCurrentChapter(currentChapter - 1);
      setCurrentSection(chapters[currentChapter - 1].sections.length - 1);
      window.scrollTo(0, 0);
    }
  };
  
  const goToSection = (sectionIndex: number) => {
    setCurrentSection(sectionIndex);
    window.scrollTo(0, 0);
  };

  const currentChapterObj = chapters[currentChapter];
  const currentSectionObj = currentChapterObj.sections[currentSection];
  
  return (
    <div className="extended-guide">
      <div className="guide-header">
        <button className="close-guide" onClick={onClose}>
          <svg viewBox="0 0 24 24" width="24" height="24">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
          </svg>
        </button>
        <h1>Papin: The Complete Guide</h1>
        <div className="guide-subtitle">From Beginner to Expert</div>
      </div>
      
      <div className="guide-navigation">
        <div className="chapter-navigation">
          <button 
            className="chapter-nav-button prev-chapter" 
            onClick={prevChapter}
            disabled={currentChapter === 0}
          >
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
            </svg>
            Previous Chapter
          </button>
          <div className="chapter-select">
            <select 
              value={currentChapter}
              onChange={(e) => {
                setCurrentChapter(parseInt(e.target.value));
                setCurrentSection(0);
                window.scrollTo(0, 0);
              }}
            >
              {chapters.map((chapter, index) => (
                <option key={index} value={index}>
                  {chapter.title}
                </option>
              ))}
            </select>
          </div>
          <button 
            className="chapter-nav-button next-chapter" 
            onClick={nextChapter}
            disabled={currentChapter === chapters.length - 1}
          >
            Next Chapter
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" />
            </svg>
          </button>
        </div>
      </div>
      
      <div className="guide-content">
        <div className="section-sidebar">
          <h2>{currentChapterObj.title}</h2>
          <ul className="section-list">
            {currentChapterObj.sections.map((section, index) => (
              <li 
                key={index} 
                className={index === currentSection ? 'active' : ''}
                onClick={() => goToSection(index)}
              >
                {section.title}
              </li>
            ))}
          </ul>
        </div>
        
        <div className="section-content">
          <div className="section-header">
            <h2>{currentSectionObj.title}</h2>
            <div className="progress-indicator">
              Section {currentSection + 1} of {currentChapterObj.sections.length}
            </div>
          </div>
          
          <div className="section-body">
            {currentSectionObj.content}
          </div>
          
          <div className="section-navigation">
            <button 
              className="section-nav-button prev-section" 
              onClick={prevSection}
              disabled={currentChapter === 0 && currentSection === 0}
            >
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
              </svg>
              Previous
            </button>
            <button 
              className="section-nav-button next-section" 
              onClick={nextSection}
              disabled={currentChapter === chapters.length - 1 && currentSection === currentChapterObj.sections.length - 1}
            >
              Next
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ExtendedGuide;
