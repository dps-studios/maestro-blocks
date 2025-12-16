/**
 * EditableHeader component
 * 
 * A markdown-based editable header for worksheets.
 * - View mode: renders markdown as HTML
 * - Edit mode: textarea for raw markdown input
 * - Alignment controls visible on hover/focus
 */

import { createSignal, Show } from 'solid-js';
import snarkdown from 'snarkdown';
import { scoreStore } from '../stores/score';
import type { HeaderAlignment } from '../types/score';

const DEFAULT_PLACEHOLDER = `# Worksheet Title
_Subtitle or additional info_`;

export function EditableHeader() {
  const [isEditing, setIsEditing] = createSignal(false);
  const [isHovered, setIsHovered] = createSignal(false);
  let textareaRef: HTMLTextAreaElement | undefined;

  const metadata = () => scoreStore.state.metadata;
  const content = () => metadata().headerContent || '';
  const alignment = () => metadata().alignment || 'start';

  // Render markdown to HTML
  const renderedContent = () => {
    const raw = content();
    if (!raw) {
      // Show placeholder in muted style
      return snarkdown(DEFAULT_PLACEHOLDER);
    }
    return snarkdown(raw);
  };

  const handleStartEdit = () => {
    setIsEditing(true);
    // Focus textarea after it renders
    requestAnimationFrame(() => {
      if (textareaRef) {
        textareaRef.focus();
        // If empty, populate with placeholder for convenience
        if (!content()) {
          textareaRef.value = DEFAULT_PLACEHOLDER;
          textareaRef.select();
        }
      }
    });
  };

  const handleStopEdit = () => {
    setIsEditing(false);
  };

  const handleTextareaChange = (e: Event) => {
    const target = e.target as HTMLTextAreaElement;
    scoreStore.updateMetadata({ headerContent: target.value });
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      handleStopEdit();
    }
  };

  const setAlignment = (align: HeaderAlignment) => {
    scoreStore.updateMetadata({ alignment: align });
  };

  const alignmentClass = () => `worksheet-header--align-${alignment()}`;

  return (
    <div
      class={`worksheet-header ${alignmentClass()} ${isEditing() ? 'worksheet-header--editing' : ''}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Alignment controls - visible on hover or when editing */}
      <Show when={isHovered() || isEditing()}>
        <div class="worksheet-header__controls">
          <button
            type="button"
            class={`worksheet-header__align-btn ${alignment() === 'start' ? 'worksheet-header__align-btn--active' : ''}`}
            onClick={() => setAlignment('start')}
            title="Align left"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="2" y="3" width="12" height="2" />
              <rect x="2" y="7" width="8" height="2" />
              <rect x="2" y="11" width="10" height="2" />
            </svg>
          </button>
          <button
            type="button"
            class={`worksheet-header__align-btn ${alignment() === 'center' ? 'worksheet-header__align-btn--active' : ''}`}
            onClick={() => setAlignment('center')}
            title="Align center"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="2" y="3" width="12" height="2" />
              <rect x="4" y="7" width="8" height="2" />
              <rect x="3" y="11" width="10" height="2" />
            </svg>
          </button>
          <button
            type="button"
            class={`worksheet-header__align-btn ${alignment() === 'end' ? 'worksheet-header__align-btn--active' : ''}`}
            onClick={() => setAlignment('end')}
            title="Align right"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="2" y="3" width="12" height="2" />
              <rect x="6" y="7" width="8" height="2" />
              <rect x="4" y="11" width="10" height="2" />
            </svg>
          </button>
        </div>
      </Show>

      <Show
        when={!isEditing()}
        fallback={
          <textarea
            ref={textareaRef}
            class="worksheet-header__textarea"
            value={content()}
            onInput={handleTextareaChange}
            onBlur={handleStopEdit}
            onKeyDown={handleKeyDown}
            placeholder={DEFAULT_PLACEHOLDER}
            rows={4}
          />
        }
      >
        <div
          class={`worksheet-header__content ${!content() ? 'worksheet-header__content--placeholder' : ''}`}
          onClick={handleStartEdit}
          innerHTML={renderedContent()}
        />
      </Show>
    </div>
  );
}
