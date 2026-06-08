'use client';
import { Search, Settings, Bell, CircleUser, Brain } from 'lucide-react';

export default function Header() {
  return (
    <header className="bg-surface-dim text-primary font-headline-md font-label-md border-b border-outline-variant flex justify-between items-center w-full px-lg py-sm h-16 shrink-0 z-50">
      <div className="flex items-center gap-md">
        <Brain className="w-7 h-7" />
        <span className="font-headline-md text-headline-md font-bold text-primary hidden sm:inline-block">Axeane Automation Sync</span>
      </div>
      <div className="flex-1 max-w-xl mx-xl">
        <div className="relative flex items-center group">
          <Search className="absolute left-sm text-outline w-5 h-5 pointer-events-none group-focus-within:text-primary transition-colors" />
          <input 
            className="w-full bg-surface-container border border-outline-variant rounded-lg pl-xl pr-md py-xs font-label-md text-on-surface focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/20 transition-all" 
            placeholder="Search sequences, metrics, parameters..." 
            type="text" 
          />
        </div>
      </div>
      <div className="flex items-center gap-md">
        <span className="font-code-sm text-code-sm text-on-surface-variant opacity-60 hidden md:inline-block border border-outline-variant/50 px-2 py-0.5 rounded">v2.4.0-stable</span>
        <button className="p-sm rounded-full hover:bg-surface-container-high transition-colors duration-200 active:scale-95 text-on-surface-variant hover:text-on-surface">
          <Settings className="w-5 h-5" />
        </button>
        <button className="p-sm rounded-full hover:bg-surface-container-high transition-colors duration-200 active:scale-95 relative text-on-surface-variant hover:text-on-surface">
          <Bell className="w-5 h-5" />
          <span className="absolute top-2 right-2 w-2 h-2 bg-secondary rounded-full"></span>
        </button>
        <button className="p-sm rounded-full hover:bg-surface-container-high transition-colors duration-200 active:scale-95 text-primary">
          <CircleUser className="w-6 h-6" />
        </button>
      </div>
    </header>
  );
}
