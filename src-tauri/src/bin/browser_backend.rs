use crate::app_state::{EditableRow, FormatTemplate};
use evalexpr::ContextWithMutableVariables;

pub fn compile_payload_from_state(rows: &[EditableRow], templates: &[FormatTemplate]) -> String {
    let mut payload_array = Vec::new();

    let default_template = templates.iter()
        .find(|t| t.client_key == "DEFAULT")
        .or_else(|| templates.first());

    let default_template = match default_template {
        Some(t) => t,
        None => return "[]".to_string(),
    };

    for r in rows {
        let matched_rule = templates.iter()
            .find(|t| r.client.to_uppercase().contains(&t.client_key.to_uppercase()))
            .unwrap_or(default_template);

        let ttc_num: f64 = r.ttc.replace(',', ".").parse().unwrap_or(0.0);
        let ht_num: f64 = r.ht.replace(',', ".").parse().unwrap_or(0.0);
        let tva_num: f64 = r.tva.replace(',', ".").parse().unwrap_or(0.0);

        let mut context = evalexpr::HashMapContext::new();
        context.set_value("row.ttc".into(), evalexpr::Value::from(ttc_num)).unwrap();
        context.set_value("row.ht".into(), evalexpr::Value::from(ht_num)).unwrap();
        context.set_value("row.tva".into(), evalexpr::Value::from(tva_num)).unwrap();

        let mut computed_lignes = Vec::new();
        for t_row in &matched_rule.rows {
            let amount = evalexpr::eval_with_context(&t_row.formula, &context)
                .unwrap_or(evalexpr::Value::from(0.0))
                .as_number()
                .unwrap_or(0.0);

            computed_lignes.push(serde_json::json!({
                "compte": t_row.compte,
                "type": if t_row.entry_type == crate::app_state::EntryType::Debit { "debit" } else { "credit" },
                "amount": amount
            }));
        }

        payload_array.push(serde_json::json!({
            "client": r.client,
            "ref": r.reference,
            "date": r.date,
            "lignes": computed_lignes
        }));
    }

    serde_json::to_string(&payload_array).unwrap()
}

pub fn compile_page_detection_script() -> String {
    r#"
    (function() {
        const root = document.querySelector('.td-root');
        if (!root) return "NO_FORM";

        const form = document.getElementById('ecritureForm');
        if (!form) return "NO_FORM";

        const hasDateInput    = !!document.getElementById('ec-date-creation');
        const hasJournalSel   = !!document.getElementById('jo-eav');
        const hasPieceInput   = !!document.getElementById('idDocumentInputMD2');
        const hasLibelleInput = !!document.getElementById('inputLibelleIdMD2');
        const hasLineInput    = !!document.getElementById('cc_0_3');

        if (hasDateInput && hasJournalSel && hasPieceInput) {
            try {
                const $scope = angular.element(root).scope();
                if (!$scope || !$scope.ecritureGrouping) return "WRONG_PAGE";
            } catch(e) {
                return "WRONG_PAGE";
            }
            return "READY";
        }

        return "WRONG_PAGE";
    })();
    "#.to_string()
}

pub fn build_injection_script(json_payload: &str, journal_code: &str) -> String {
    format!(
        r#"(function() {{
    'use strict';

    const payloadData = {payload};
    const journalCode = '{journal}';

    if (!payloadData || payloadData.length === 0) {{
        console.warn('[Axeane Bridge] Empty payload.');
        return;
    }}

    const rootEl = document.querySelector('.td-root');
    if (!rootEl) {{ alert('[Axeane Bridge] Not on Saisie des ecritures page.'); return; }}
    const $scope = angular.element(rootEl).scope();
    if (!$scope) {{ alert('[Axeane Bridge] Angular scope not found.'); return; }}

    function apply(fn) {{ $scope.$apply(fn); }}

    function resolveJournal() {{
        const eid = $scope.contextComptable &&
            $scope.contextComptable.currentEntreprise &&
            $scope.contextComptable.currentEntreprise.entrepriseId;
        if (!eid) return null;
        const map = $scope.model && $scope.model.mapCodeJournauxEntreprise;
        if (!map || !map[eid]) return null;
        return map[eid].find(j => j.code === journalCode) || null;
    }}

    function resolveCompte(compte) {{
        const eid = $scope.contextComptable &&
            $scope.contextComptable.currentEntreprise &&
            $scope.contextComptable.currentEntreprise.entrepriseId;
        if (!eid) return null;
        const map = $scope.model &&
            $scope.model.mapComptesComptableEntreprise &&
            $scope.model.mapComptesComptableEntreprise[eid];
        if (!map) return null;
        return map.find(c =>
            c.compteComptable === compte ||
            (c.dernierCompteLibelle && c.dernierCompteLibelle.startsWith(compte))
        ) || null;
    }}

    let invoiceIdx = 0;

    function nextInvoice() {{
        if (invoiceIdx >= payloadData.length) {{
            console.log('[Axeane Bridge] Done - ' + payloadData.length + ' invoices injected.');
            return;
        }}
        const entry = payloadData[invoiceIdx];
        console.log('[Bridge] Invoice ' + (invoiceIdx+1) + '/' + payloadData.length + ' - ' + entry.ref);

        apply(function() {{
            if (typeof $scope.resetEcritures === 'function') $scope.resetEcritures();
        }});

        setTimeout(function() {{
            apply(function() {{
                $scope.ecritureGrouping.dateOperation = entry.date;
                $scope.ecritureGrouping.piece         = entry.ref;
                $scope.ecritureGrouping.libelle       = ('FACTURE ' + entry.ref + ' ' + entry.client)
                                                          .toUpperCase().substring(0, 120);
                const j = resolveJournal();
                if (j) {{
                    $scope.ecritureGrouping.journal = j;
                    if (typeof $scope.JournalCodeChanges === 'function') $scope.JournalCodeChanges();
                }} else {{
                    console.warn('[Bridge] Journal not found: ' + journalCode);
                }}
            }});

            setTimeout(function() {{ fillLines(entry, 0, function() {{
                apply(function() {{
                    if (typeof $scope.saveEcriture === 'function') $scope.saveEcriture();
                }});
                invoiceIdx++;
                setTimeout(nextInvoice, 2200);
            }}); }}, 400);
        }}, 300);
    }}

    function fillLines(entry, lineIdx, onDone) {{
        if (lineIdx >= entry.lignes.length) {{ onDone(); return; }}
        const ligne = entry.lignes[lineIdx];

        apply(function() {{
            const existing = ($scope.ecritureGrouping.ecritureComptables || []).length;
            if (existing <= lineIdx) {{
                if (typeof $scope.ajouterEcriture === 'function') $scope.ajouterEcriture();
            }}
        }});

        setTimeout(function() {{
            apply(function() {{
                const ecs = $scope.ecritureGrouping.ecritureComptables;
                if (!ecs || !ecs[lineIdx]) {{
                    console.warn('[Bridge] Missing line slot ' + lineIdx);
                    fillLines(entry, lineIdx + 1, onDone);
                    return;
                }}
                const ec = ecs[lineIdx];

                const compteObj = resolveCompte(ligne.compte);
                if (compteObj) {{
                    ec.comptesComptable = compteObj.dernierCompteLibelle || ligne.compte;
                    if (typeof $scope.onSelectCompteComptable === 'function') {{
                        $scope.onSelectCompteComptable(
                            compteObj, compteObj.dernierCompteLibelle,
                            compteObj.dernierCompteLibelle, lineIdx, ec
                        );
                        if (typeof $scope.noNeedTreasuryOperation === 'function')
                            $scope.noNeedTreasuryOperation(ec, lineIdx);
                    }}
                }} else {{
                    ec.comptesComptable = ligne.compte;
                }}

                ec.extraLibelle = ('FAC ' + entry.ref).toUpperCase().substring(0, 40);

                const amt = ligne.amount.toFixed(3);
                if (ligne.type === 'debit') {{
                    ec.debit  = amt;
                    ec.credit = '0,000';
                }} else {{
                    ec.credit = amt;
                    ec.debit  = '0,000';
                }}

                if (typeof $scope.calculateTotalDebit  === 'function') $scope.calculateTotalDebit(true, ec, false);
                if (typeof $scope.calculateTotalCredit === 'function') $scope.calculateTotalCredit(true, ec, false);
            }});

            setTimeout(function() {{ fillLines(entry, lineIdx + 1, onDone); }}, 350);
        }}, 250);
    }}

    nextInvoice();
}})();"#,
        payload = json_payload,
        journal = journal_code.replace('\\', "\\\\").replace('\'', "\\'"),
    )
}
