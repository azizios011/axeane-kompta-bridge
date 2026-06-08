import { Dispatch, SetStateAction } from 'react';
import { Code2, CopyPlus, FunctionSquare, Plus, Trash2 } from 'lucide-react';
import { AppState, createTemplate, createTemplateRow } from '@/lib/app-state';

interface FormulaSetupProps { appState: AppState; setAppState: Dispatch<SetStateAction<AppState>>; }

export default function FormulaSetup({ appState, setAppState }: FormulaSetupProps) {
  const updateTemplate = (templateIndex: number, clientKey: string) => setAppState((current) => ({ ...current, templates: current.templates.map((template, index) => index === templateIndex ? { ...template, clientKey } : template) }));
  const updateRow = (templateIndex: number, rowIndex: number, field: 'compte' | 'entryType' | 'formula', value: string) => setAppState((current) => ({ ...current, templates: current.templates.map((template, index) => index === templateIndex ? { ...template, rows: template.rows.map((row, currentRowIndex) => currentRowIndex === rowIndex ? { ...row, [field]: value } : row) } : template) }));

  return (
    <div className="flex-1 overflow-y-auto w-full h-full p-lg bg-background animate-fade-in relative z-10">
      <div className="max-w-container-max mx-auto space-y-lg pb-12">
        <section className="flex flex-col md:flex-row md:items-end justify-between gap-md mb-xl"><div className="space-y-sm"><h1 className="font-headline-lg text-headline-lg text-on-surface">Formula Rules &amp; Ledger Mappings Manager</h1><p className="text-on-surface-variant max-w-2xl font-body-md">Map matching criteria and dynamic string scripts using variables: row.ttc, row.ht, row.tva.</p></div><button onClick={() => setAppState((current) => ({ ...current, templates: [...current.templates, createTemplate()] }))} className="flex items-center gap-sm bg-primary-container text-on-primary-container px-lg py-sm font-label-md text-label-md font-bold rounded-lg hover:brightness-110 transition-all active:scale-95 shadow-lg shadow-primary-container/20"><CopyPlus className="w-[18px] h-[18px]" />Generate Client Template Mapping Card</button></section>
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-lg">
          {appState.templates.map((template, templateIndex) => (
            <article key={templateIndex} className="bg-surface-container-low border border-outline-variant rounded-lg p-lg flex flex-col gap-md hover:border-outline transition-colors group relative overflow-hidden">
              <div className="absolute top-0 left-0 w-1 h-full bg-secondary" />
              <div className="flex justify-between items-start"><div className="flex items-center gap-sm"><Code2 className="text-secondary w-5 h-5" /><h4 className="font-headline-md text-label-md text-on-surface uppercase tracking-wider">Client Rule: {template.id ?? `NEW_${templateIndex + 1}`}</h4></div><button onClick={() => setAppState((current) => ({ ...current, templates: current.templates.filter((_, index) => index !== templateIndex) }))} className="text-on-surface-variant hover:text-error transition-colors p-xs" title="Remove template"><Trash2 className="w-5 h-5" /></button></div>
              <div className="space-y-xs"><label className="font-label-md text-code-sm text-outline uppercase">Identity Matching Keyword</label><input value={template.clientKey} onChange={(event) => updateTemplate(templateIndex, event.target.value)} className="w-full bg-surface-container-high border border-outline-variant rounded p-sm font-label-md text-label-md focus:border-primary focus:ring-0 text-on-surface outline-none" type="text" /></div>
              <div className="space-y-sm border-t border-outline-variant/30 pt-md">
                {template.rows.map((row, rowIndex) => (
                  <div key={rowIndex} className="grid grid-cols-12 gap-sm items-center">
                    <input aria-label="Compte" value={row.compte} onChange={(event) => updateRow(templateIndex, rowIndex, 'compte', event.target.value)} className="col-span-12 md:col-span-3 bg-surface-container-lowest border border-outline-variant rounded p-sm font-code-sm text-code-sm text-on-surface outline-none" type="text" placeholder="Compte" />
                    <select aria-label="Entry type" value={row.entryType} onChange={(event) => updateRow(templateIndex, rowIndex, 'entryType', event.target.value)} className="col-span-5 md:col-span-2 bg-surface-container-lowest border border-outline-variant rounded p-sm font-code-sm text-code-sm text-on-surface outline-none"><option value="debit">Debit</option><option value="credit">Credit</option></select>
                    <div className="col-span-7 md:col-span-6 relative"><FunctionSquare className="absolute left-sm top-1/2 -translate-y-1/2 text-primary w-4 h-4" /><input aria-label="Formula" value={row.formula} onChange={(event) => updateRow(templateIndex, rowIndex, 'formula', event.target.value)} className="w-full bg-surface-container-lowest border border-outline-variant rounded pl-xl p-sm font-code-sm text-code-sm focus:border-primary focus:ring-0 text-secondary outline-none" type="text" /></div>
                    <button onClick={() => setAppState((current) => ({ ...current, templates: current.templates.map((item, index) => index === templateIndex ? { ...item, rows: item.rows.filter((_, currentRowIndex) => currentRowIndex !== rowIndex) } : item) }))} className="col-span-12 md:col-span-1 text-error justify-self-end p-xs" title="Remove ledger row"><Trash2 className="w-5 h-5" /></button>
                  </div>
                ))}
              </div>
              <button onClick={() => setAppState((current) => ({ ...current, templates: current.templates.map((item, index) => index === templateIndex ? { ...item, rows: [...item.rows, createTemplateRow()] } : item) }))} className="self-start inline-flex items-center gap-sm bg-surface-container-high border border-outline-variant px-md py-sm rounded font-label-md text-label-md hover:bg-surface-variant transition-all"><Plus className="w-4 h-4" />Add Ledger Row</button>
            </article>
          ))}
        </div>
      </div>
    </div>
  );
}
