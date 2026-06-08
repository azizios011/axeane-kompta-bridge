import { Dispatch, SetStateAction } from 'react';
import { FileText, Filter, Plus, Rocket, Trash2 } from 'lucide-react';
import { AppState, createEmptyRow, formatAmount, totalsForRows } from '@/lib/app-state';

interface DataTableEditorProps { appState: AppState; setAppState: Dispatch<SetStateAction<AppState>>; patchState: (patch: Partial<AppState>) => void; }

export default function DataTableEditor({ appState, setAppState, patchState }: DataTableEditorProps) {
  const totals = totalsForRows(appState.csvRows);
  const updateRow = (index: number, field: keyof AppState['csvRows'][number], value: string) => setAppState((current) => ({ ...current, csvRows: current.csvRows.map((row, rowIndex) => rowIndex === index ? { ...row, [field]: value } : row) }));
  const importPdf = () => {
    patchState({ isLoading: true, status: 'Initializing non-blocking background runtime parser token pipelines...' });
    window.setTimeout(() => setAppState((current) => ({ ...current, isLoading: false, status: 'Frontend is ready for the Rust PDF/AI parser bridge. Connect the backend endpoint to populate document.pdf rows.' })), 900);
  };
  const toggleAutoDetect = () => {
    if (appState.csvRows.length === 0) { patchState({ status: 'Error: Cannot start automation tracking with an empty spreadsheet grid.' }); return; }
    const nextValue = !appState.autoDetectActive;
    patchState({ autoDetectActive: nextValue, status: nextValue ? 'Automation listener armed. Navigate to your Axeane ledger sheet to begin.' : 'Auto-detection monitoring suspended manually.' });
  };

  return (
    <div className="flex-1 overflow-hidden flex flex-col bg-background p-lg animate-fade-in relative z-10 w-full h-full">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-md mb-lg shrink-0">
        <div><h2 className="font-headline-lg text-headline-lg text-on-background">Extracted Row Items Spreadsheet Grid</h2><p className="text-on-surface-variant font-body-md">Modify parameters inside cells dynamically before executing browser delivery macros.</p></div>
        <div className="flex flex-wrap gap-sm">
          <button onClick={importPdf} className="bg-primary-container text-on-primary-container px-md py-sm rounded flex items-center gap-sm font-label-md text-label-md hover:brightness-110 active:scale-[0.98] transition-all"><FileText className="w-[18px] h-[18px]" />Import document.pdf with Dynamic LLM</button>
          <button onClick={() => setAppState((current) => ({ ...current, csvRows: [...current.csvRows, createEmptyRow()] }))} className="bg-surface-container-high text-on-surface border border-outline-variant px-md py-sm rounded flex items-center gap-sm font-label-md text-label-md hover:bg-surface-container-highest transition-colors"><Plus className="w-[18px] h-[18px]" />Insert Custom Row</button>
          <button onClick={toggleAutoDetect} className={`${appState.autoDetectActive ? 'bg-tertiary-container text-on-tertiary-container' : 'bg-secondary-container text-on-secondary-container'} px-md py-sm rounded flex items-center gap-sm font-label-md text-label-md font-bold hover:brightness-110 active:scale-[0.98] transition-all`}><Rocket className="w-[18px] h-[18px]" />{appState.autoDetectActive ? 'Monitoring Browser... Stop' : 'Start Auto-Detection Filling'}</button>
        </div>
      </div>
      <div className="flex-1 bg-surface-container-low rounded-lg border border-outline-variant overflow-hidden flex flex-col min-h-0">
        <div className="p-md border-b border-outline-variant flex items-center justify-between bg-surface-container-lowest/50 shrink-0"><div className="flex gap-md"><div className="flex items-center gap-sm text-on-surface-variant text-code-sm font-code-sm border-r border-outline-variant pr-md"><Filter className="w-[16px] h-[16px]" />Filter: All Rows</div><div className="text-on-surface-variant text-code-sm font-code-sm">Showing <span className="text-primary font-bold">{appState.csvRows.length}</span> entries</div></div></div>
        {appState.isLoading ? <div className="flex-1 grid place-items-center text-center text-on-surface-variant"><div><div className="mx-auto mb-md h-12 w-12 animate-spin rounded-full border-2 border-primary border-t-transparent" /><p className="text-primary font-label-md">Isolated AI backend ripping ledger structures... Parsing rows downstream.</p></div></div> : (
          <div className="flex-1 overflow-auto w-full"><table className="w-full text-left border-collapse min-w-[860px]"><thead className="sticky top-0 bg-surface-container-high z-10 shadow-[0_1px_0_var(--color-outline-variant)]"><tr className="text-on-surface-variant font-label-md text-label-md"><th className="py-md px-md font-medium whitespace-nowrap">Client Match Key</th><th className="py-md px-md font-medium whitespace-nowrap">Invoice Ref</th><th className="py-md px-md font-medium whitespace-nowrap">Op Date</th><th className="py-md px-md font-medium text-right whitespace-nowrap">Total TTC</th><th className="py-md px-md font-medium text-right whitespace-nowrap">Base HT</th><th className="py-md px-md font-medium text-right whitespace-nowrap">VAT Amount</th><th className="py-md px-md font-medium text-center whitespace-nowrap">Actions</th></tr></thead><tbody className="font-label-md text-label-md text-on-surface">
            {appState.csvRows.map((row, index) => <tr key={index} className="border-b border-outline-variant/30 hover:bg-surface-container-highest/30 transition-colors group">{(['client', 'reference', 'date', 'ttc', 'ht', 'tva'] as const).map((field) => <td key={field} className="p-sm"><input value={row[field]} onChange={(event) => updateRow(index, field, event.target.value)} className={`${field === 'ttc' || field === 'ht' || field === 'tva' ? 'text-right' : ''} w-full bg-surface-container-lowest border border-outline-variant/50 rounded px-sm py-xs font-code-sm focus:border-primary-container focus:ring-1 focus:ring-primary-container/30 outline-none transition-all`} type="text" /></td>)}<td className="p-sm text-center"><button onClick={() => setAppState((current) => ({ ...current, csvRows: current.csvRows.filter((_, rowIndex) => rowIndex !== index) }))} className="inline-flex justify-center text-error opacity-60 group-hover:opacity-100 transition-opacity" title="Delete row"><Trash2 className="w-[18px] h-[18px]" /></button></td></tr>)}
            {appState.csvRows.length === 0 && <tr><td colSpan={7} className="p-xl text-center text-on-surface-variant font-code-sm">No rows yet. Import document.pdf or insert a custom row.</td></tr>}
          </tbody></table></div>)}
        <div className="px-md py-sm bg-surface-container-lowest/80 border-t border-outline-variant flex flex-col md:flex-row justify-between items-center gap-md shrink-0"><div className="flex gap-xl"><div className="flex flex-col"><span className="text-[10px] uppercase text-on-surface-variant font-bold">Total Batch HT</span><span className="font-code-sm text-on-surface">{formatAmount(totals.ht)}</span></div><div className="flex flex-col border-l border-outline-variant/30 pl-md"><span className="text-[10px] uppercase text-on-surface-variant font-bold">Total Batch VAT</span><span className="font-code-sm text-on-surface">{formatAmount(totals.tva)}</span></div></div><div className="flex items-center gap-md"><span className="font-headline-md text-secondary font-bold">{formatAmount(totals.ttc)}</span></div></div>
      </div>
    </div>
  );
}
