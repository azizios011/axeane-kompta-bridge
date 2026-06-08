'use client';
import { useState } from 'react';
import Header from '@/components/Header';
import Sidebar from '@/components/Sidebar';
import Footer from '@/components/Footer';
import DataTableEditor from '@/components/DataTableEditor';
import FormulaSetup from '@/components/FormulaSetup';
import BrowserEngine from '@/components/BrowserEngine';
import AiCoreParams from '@/components/AiCoreParams';
import { AppState, initialAppState } from '@/lib/app-state';

export default function Home() {
  const [activeTab, setActiveTab] = useState('table');
  const [appState, setAppState] = useState<AppState>(initialAppState);
  const patchState = (patch: Partial<AppState>) => setAppState((current) => ({ ...current, ...patch }));

  return (
    <div className="flex flex-col h-screen w-full overflow-hidden bg-background text-on-background">
      <Header />
      <div className="flex flex-1 overflow-hidden min-h-0 relative">
        <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} onTriggerSync={() => setActiveTab('table')} />
        <main className="flex-1 flex flex-col min-w-0 relative h-full">
          {activeTab === 'table' && <DataTableEditor appState={appState} setAppState={setAppState} patchState={patchState} />}
          {activeTab === 'formula' && <FormulaSetup appState={appState} setAppState={setAppState} />}
          {activeTab === 'browser' && <BrowserEngine appState={appState} patchState={patchState} />}
          {activeTab === 'core' && <AiCoreParams appState={appState} patchState={patchState} />}
        </main>
      </div>
      <Footer status={appState.status} />
    </div>
  );
}
