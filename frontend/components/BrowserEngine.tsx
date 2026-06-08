import { Copy, FolderOpen, Globe, Play, Terminal, Tv } from 'lucide-react';
import { AppState } from '@/lib/app-state';
interface BrowserEngineProps { appState: AppState; patchState: (patch: Partial<AppState>) => void; }
export default function BrowserEngine({ appState, patchState }: BrowserEngineProps) {
  const initializeBrowser = () => patchState({ status: `Browser launch requested for ${appState.browserPath || 'chrome'} on debug port ${appState.port}. Rust subprocess launch bridge required for native execution.` });
  return <div className="flex-1 w-full h-full flex flex-col relative bg-background overflow-y-auto pb-xl p-lg animate-fade-in z-10"><div className="grid grid-cols-12 gap-lg max-w-container-max mx-auto w-full"><div className="col-span-12 mb-md"><h2 className="text-display-lg font-display-lg text-on-surface">Browser Engine</h2></div><section className="col-span-12 lg:col-span-5 flex flex-col gap-lg"><div className="bg-surface-container border border-outline-variant p-lg rounded-lg shadow-sm"><div className="flex items-center gap-sm mb-lg border-b border-outline-variant pb-md"><Tv className="text-primary w-6 h-6" /><h3 className="text-headline-md font-headline-md">Remote Proxy Interface Settings</h3></div><div className="space-y-lg"><label className="block"><span className="font-label-md text-label-md text-on-surface-variant mb-sm block">Debugging Subprocess Pipeline Socket Port</span><input value={appState.port} onChange={(event) => patchState({ port: event.target.value })} className="w-full bg-surface-container-lowest border border-outline-variant text-on-surface font-label-md p-md rounded outline-none" type="text" /></label><label className="block"> 
  <span className="font-label-md text-label-md text-on-surface-variant mb-sm block">Browser Executable Path</span> 
  <div className="flex gap-sm"> 
    <input 
      value={appState.browserPath} 
      onChange={(event) => patchState({ browserPath: event.target.value })} 
      className="flex-1 bg-surface-container-lowest border border-outline-variant text-on-surface font-label-md p-md rounded outline-none" 
      type="text" 
      placeholder="e.g. chrome or C:\Program Files\..." 
    /> 
    <button 
      onClick={async () => { 
        const { open } = await import('@tauri-apps/plugin-dialog'); 
        const selected = await open({ 
          title: 'Select Browser Executable', 
          filters: [{ name: 'Executable', extensions: ['exe'] }], 
        }); 
        if (typeof selected === 'string') patchState({ browserPath: selected }); 
      }} 
      className="bg-surface-container-highest border border-outline-variant hover:border-secondary text-on-surface px-md rounded transition-all flex items-center gap-xs" 
      title="Browse" 
    > 
      <FolderOpen className="w-4 h-4" /> 
    </button> 
  </div> 
</label><label className="flex items-center justify-between bg-surface-container-low p-md rounded border border-outline-variant/50 cursor-pointer"><span><span className="font-label-md text-label-md text-on-surface block">Launch Private / Incognito Session Window Sandbox</span><span className="text-code-sm font-code-sm text-on-surface-variant opacity-70">Prevent session persistence across subprocesses.</span></span><input checked={appState.incognito} onChange={(event) => patchState({ incognito: event.target.checked })} className="h-5 w-5 accent-secondary" type="checkbox" /></label><button onClick={initializeBrowser} className="w-full bg-primary-container hover:bg-primary-container/90 text-on-primary-container font-label-md text-label-md font-bold py-lg rounded flex items-center justify-center gap-md active:scale-95 transition-all shadow-lg shadow-primary-container/20 mt-xl"><Play className="w-5 h-5 fill-current" />Initialize Isolated Chromium Subprocess instance</button></div></div></section><section className="col-span-12 lg:col-span-7 flex flex-col gap-lg"><div className="bg-surface-container border border-outline-variant p-lg rounded-lg shadow-sm flex-1 flex flex-col"><div className="flex items-center justify-between mb-lg border-b border-outline-variant pb-md"><div className="flex items-center gap-sm"><Globe className="text-secondary w-6 h-6" /><h3 className="text-headline-md font-headline-md">Execution Gateway Controls</h3></div><span className="text-code-sm font-code-sm text-on-surface-variant bg-surface-container-highest px-sm py-1 rounded">127.0.0.1:8085</span></div><div className="bg-surface-container-lowest border border-outline-variant rounded p-lg mb-lg"><label className="text-code-sm font-code-sm text-outline block mb-xs">WebSocket Pipe Gateway Core</label><div className="flex items-center gap-md"><span className="text-headline-sm font-headline-sm text-secondary font-code-sm text-lg">127.0.0.1:8085</span><Copy className="w-4 h-4 text-outline" /></div></div><button onClick={() => patchState({ triggerInjection: true, status: 'Injection transaction block macros dispatched downstream.' })} className="group w-full bg-surface-container-highest border border-outline hover:border-secondary hover:bg-surface-container-highest/80 text-on-surface font-label-md text-label-md font-bold py-lg rounded flex items-center justify-center gap-md active:translate-y-1 transition-all"><Terminal className="w-5 h-5 group-hover:text-secondary transition-colors" />Dispatch Parsed Matrix Values Downstream</button></div></section></div></div>;
}
