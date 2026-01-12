import { emit } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { InternalEvent, ShowToastRequest } from '@yaakapp-internal/plugins';
import { openSettings } from '../commands/openSettings';
import { Button } from '../components/core/Button';
import { ButtonInfiniteLoading } from '../components/core/ButtonInfiniteLoading';
import { Icon } from '../components/core/Icon';
import { HStack, VStack } from '../components/core/Stacks';

// Listen for toasts
import { listenToTauriEvent } from '../hooks/useListenToTauriEvent';
import { updateAvailableAtom } from './atoms';
import { stringToColor } from './color';
import { generateId } from './generateId';
import { jotaiStore } from './jotai';
import { showPrompt } from './prompt';
import { showPromptForm } from './prompt-form';
import { invokeCmd } from './tauri';
import { showToast } from './toast';

export function initGlobalListeners() {
  listenToTauriEvent<ShowToastRequest>('show_toast', (event) => {
    showToast({ ...event.payload });
  });

  listenToTauriEvent('settings', () => openSettings.mutate(null));

  // Listen for plugin events
  listenToTauriEvent<InternalEvent>('plugin_event', async ({ payload: event }) => {
    if (event.payload.type === 'prompt_text_request') {
      const value = await showPrompt(event.payload);
      const result: InternalEvent = {
        id: generateId(),
        replyId: event.id,
        pluginName: event.pluginName,
        pluginRefId: event.pluginRefId,
        context: event.context,
        payload: {
          type: 'prompt_text_response',
          value,
        },
      };
      await emit(event.id, result);
    } else if (event.payload.type === 'prompt_form_request') {
      const values = await showPromptForm({
        id: event.payload.id,
        title: event.payload.title,
        description: event.payload.description,
        inputs: event.payload.inputs,
        confirmText: event.payload.confirmText,
        cancelText: event.payload.cancelText,
      });
      const result: InternalEvent = {
        id: generateId(),
        replyId: event.id,
        pluginName: event.pluginName,
        pluginRefId: event.pluginRefId,
        context: event.context,
        payload: {
          type: 'prompt_form_response',
          values,
        },
      };
      await emit(event.id, result);
    }
  });
}
