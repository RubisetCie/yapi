import { setWindowTitle } from '@yaakapp-internal/mac-window';
import { useAtomValue } from 'jotai';
import { useEffect } from 'react';
import { appInfo } from '../lib/appInfo';
import { resolvedModelName } from '../lib/resolvedModelName';
import { useActiveEnvironment } from './useActiveEnvironment';
import { activeRequestAtom } from './useActiveRequest';
import { activeWorkspaceAtom } from './useActiveWorkspace';

export function useSyncWorkspaceRequestTitle() {
  const activeWorkspace = useAtomValue(activeWorkspaceAtom);
  const activeEnvironment = useActiveEnvironment();
  const activeRequest = useAtomValue(activeRequestAtom);

  useEffect(() => {
    let newTitle = activeWorkspace ? activeWorkspace.name : 'Yapi';
    if (activeEnvironment) {
      newTitle += ` [${activeEnvironment.name}]`;
    }
    if (activeRequest) {
      newTitle += ` › ${resolvedModelName(activeRequest)}`;
    }

    if (appInfo.isDev) {
      newTitle = `[DEV] ${newTitle}`;
    }

    setWindowTitle(newTitle);
  }, [activeEnvironment, activeRequest, activeWorkspace]);
}
