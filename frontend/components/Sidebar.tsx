'use client';
import { Table, FunctionSquare, Globe, Cpu, RefreshCw, FileText, HelpCircle, Terminal } from 'lucide-react';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  onTriggerSync: () => void;
}

export default function Sidebar({ activeTab, setActiveTab, onTriggerSync }: SidebarProps) {
  const navItems = [
    { id: 'table', label: 'Data Table Editor', icon: Table },
    { id: 'formula', label: 'Formula Setup', icon: FunctionSquare },
    { id: 'browser', label: 'Browser Engine', icon: Globe },
    { id: 'core', label: 'AI Core Params', icon: Cpu },
  ];

  return (
    <aside className="bg-surface-container-low text-secondary border-r border-outline-variant flex flex-col h-full w-64 shrink-0 z-40">
      <div className="px-md mt-lg mb-xl flex items-center gap-sm">
        <div className="w-10 h-10 rounded bg-primary/10 flex items-center justify-center border border-primary/20 shrink-0">
          <Terminal className="text-primary w-5 h-5" />
        </div>
        <div>
          <h3 className="font-headline-md text-label-md text-on-surface font-bold leading-none mb-1">Operator Console</h3>
          <p className="font-code-sm text-code-sm text-secondary uppercase tracking-widest text-[10px]">Active Session</p>
        </div>
      </div>
      
      <nav className="flex-1 px-sm space-y-1 overflow-y-auto custom-scrollbar">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          const Icon = item.icon;
          
          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center gap-sm px-md py-sm transition-all duration-150 rounded-lg group text-left ${
                isActive 
                  ? 'bg-secondary-container text-on-secondary-container font-bold translate-x-1 shadow-sm' 
                  : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high hover:translate-x-1'
              }`}
            >
              <Icon className="w-5 h-5" />
              <span className="font-label-md text-label-md">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="px-md py-md mt-auto">
        <button onClick={onTriggerSync} className="w-full py-sm bg-primary text-on-primary font-label-md text-label-md font-bold rounded-lg hover:brightness-110 active:scale-95 transition-all flex items-center justify-center gap-xs shadow-[0_0_15px_rgba(180,197,255,0.15)]">
          <RefreshCw className="w-4 h-4" />
          Trigger Sync
        </button>
      </div>

      <div className="px-sm border-t border-outline-variant pt-md pb-6 space-y-1">
        <button className="w-full flex items-center gap-sm px-md py-xs text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high transition-all duration-150 rounded-lg text-left">
          <FileText className="w-[18px] h-[18px]" />
          <span className="font-label-md text-label-md">Docs</span>
        </button>
        <button className="w-full flex items-center gap-sm px-md py-xs text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high transition-all duration-150 rounded-lg text-left">
          <HelpCircle className="w-[18px] h-[18px]" />
          <span className="font-label-md text-label-md">Support</span>
        </button>
      </div>
    </aside>
  );
}



