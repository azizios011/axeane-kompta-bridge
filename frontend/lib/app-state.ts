export type EntryType = 'debit' | 'credit';
export interface EditableRow { client: string; reference: string; date: string; ttc: string; ht: string; tva: string; }
export interface TemplateRow { compte: string; entryType: EntryType; formula: string; }
export interface FormatTemplate { id: number | null; clientKey: string; rows: TemplateRow[]; }
export interface LlmConfig { providerName: string; endpointUrl: string; apiKey: string; modelName: string; temperature: number; maxTokens: number; }
export interface AppState { csvRows: EditableRow[]; templates: FormatTemplate[]; port: string; browserPath: string; incognito: boolean; status: string; triggerInjection: boolean; autoDetectActive: boolean; isLoading: boolean; llm: LlmConfig; }
export const defaultTemplateRows = (): TemplateRow[] => [
  { compte: '41100000', entryType: 'debit', formula: 'row.ttc' },
  { compte: '70700000', entryType: 'credit', formula: 'row.ht' },
  { compte: '43670000', entryType: 'credit', formula: 'row.tva' },
];
export const initialAppState: AppState = {
  csvRows: [],
  templates: [{ id: 1, clientKey: 'PASSAGER', rows: defaultTemplateRows() }],
  port: '9222',
  browserPath: 'chrome',
  incognito: true,
  status: 'System online. Configure parameters within AI Core tab before parsing document.pdf.',
  triggerInjection: false,
  autoDetectActive: false,
  isLoading: false,
  llm: { providerName: 'DeepSeek', endpointUrl: 'https://api.deepseek.com/v1/chat/completions', apiKey: '', modelName: 'deepseek-chat', temperature: 0.1, maxTokens: 3000 },
};
export const createEmptyRow = (): EditableRow => ({ client: 'PASSAGER', reference: 'FC/2026', date: '08/06/2026', ttc: '0.0', ht: '0.0', tva: '0.0' });
export const createTemplate = (): FormatTemplate => ({ id: null, clientKey: 'NEW COMPANY IDENTIFIER', rows: defaultTemplateRows() });
export const createTemplateRow = (): TemplateRow => ({ compte: '', entryType: 'debit', formula: '0.0' });
const parseAmount = (value: string) => Number.parseFloat(value.replace(',', '.')) || 0;
export const totalsForRows = (rows: EditableRow[]) => rows.reduce((totals, row) => ({ ht: totals.ht + parseAmount(row.ht), tva: totals.tva + parseAmount(row.tva), ttc: totals.ttc + parseAmount(row.ttc) }), { ht: 0, tva: 0, ttc: 0 });
export const formatAmount = (amount: number) => amount.toLocaleString('fr-FR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
