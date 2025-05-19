/**
 * Markdown parser utilities for the chat interface
 */
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js';

// Configure marked renderer
marked.setOptions({
  renderer: new marked.Renderer(),
  highlight: function(code, lang) {
    const language = hljs.getLanguage(lang) ? lang : 'plaintext';
    return hljs.highlight(code, { language }).value;
  },
  langPrefix: 'hljs language-',
  pedantic: false,
  gfm: true,
  breaks: true, 
  sanitize: false,
  smartypants: true,
  xhtml: false
});

// Custom renderer for code blocks to add copy button
const renderer = new marked.Renderer();
const defaultCodeRenderer = renderer.code;

renderer.code = function(code, language, isEscaped) {
  // Generate default HTML from original renderer
  const html = defaultCodeRenderer.call(this, code, language, isEscaped);
  
  // Add copy button wrapper
  return `<div class="code-block-wrapper">
    <div class="code-header">
      <span class="code-language">${language || 'text'}</span>
      <button class="copy-code-button" aria-label="Copy code">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
          <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z"/>
          <path d="M9.5 1H4a.5.5 0 0 0-.5.5v1a.5.5 0 0 0 .5.5h5.5a.5.5 0 0 0 .5-.5v-1a.5.5 0 0 0-.5-.5zm0 3h-5a.5.5 0 0 0-.5.5v1a.5.5 0 0 0 .5.5h5a.5.5 0 0 0 .5-.5v-1a.5.5 0 0 0-.5-.5z"/>
        </svg>
      </button>
    </div>
    ${html}
  </div>`;
};

marked.use({ renderer });

/**
 * Parse markdown to HTML with syntax highlighting and security measures
 */
export const parseMarkdown = (markdown: string): string => {
  // Parse markdown to HTML
  const rawHtml = marked(markdown);
  
  // Configure DOMPurify
  const purifyConfig = {
    ALLOWED_TAGS: [
      'a', 'b', 'blockquote', 'br', 'code', 'div', 'em', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'hr', 'i', 'img', 'li', 'ol', 'p', 'pre', 'span', 'strong', 'table', 'tbody',
      'td', 'th', 'thead', 'tr', 'ul', 'button', 'svg', 'path'
    ],
    ALLOWED_ATTR: [
      'href', 'src', 'alt', 'class', 'id', 'style', 'target', 'rel',
      'aria-label', 'xmlns', 'width', 'height', 'viewBox', 'fill', 'd'
    ],
    ALLOW_DATA_ATTR: false,
    ADD_ATTR: ['target'],
    ADD_TAGS: ['button', 'svg', 'path'],
    SANITIZE_DOM: true
  };
  
  // Sanitize the HTML to prevent XSS
  const cleanHtml = DOMPurify.sanitize(rawHtml, purifyConfig);
  
  return cleanHtml;
};

/**
 * Initialize copy buttons functionality
 * Call this after the markdown has been rendered
 */
export const initializeCodeCopyButtons = (): void => {
  const copyButtons = document.querySelectorAll('.copy-code-button');
  
  copyButtons.forEach(button => {
    if (button instanceof HTMLElement) {
      button.addEventListener('click', () => {
        // Find the corresponding code element
        const codeBlock = button.closest('.code-block-wrapper')?.querySelector('code');
        
        if (codeBlock) {
          // Get the text content
          const code = codeBlock.textContent || '';
          
          // Copy to clipboard
          navigator.clipboard.writeText(code).then(() => {
            // Visual feedback
            const originalInnerHTML = button.innerHTML;
            button.innerHTML = `
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path d="M12.736 3.97a.733.733 0 0 1 1.047 0c.286.289.29.756.01 1.05L7.88 12.01a.733.733 0 0 1-1.065.02L3.217 8.384a.757.757 0 0 1 0-1.06.733.733 0 0 1 1.047 0l3.052 3.093 5.4-6.425a.247.247 0 0 1 .02-.022Z"/>
              </svg>
            `;
            
            // Reset after 2 seconds
            setTimeout(() => {
              button.innerHTML = originalInnerHTML;
            }, 2000);
          }).catch(err => {
            console.error('Could not copy text: ', err);
          });
        }
      });
    }
  });
};

/**
 * Detects and formats links in plain text
 */
export const linkify = (text: string): string => {
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  return text.replace(urlRegex, (url) => {
    return `<a href="${url}" target="_blank" rel="noopener noreferrer">${url}</a>`;
  });
};