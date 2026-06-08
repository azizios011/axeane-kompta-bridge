use crate::ui::{EditableRow, FormatTemplate};
use evalexpr::ContextWithMutableVariables;

pub fn compile_payload_from_state(rows: &[EditableRow], templates: &[FormatTemplate]) -> String {
    let mut payload_array = Vec::new();

    let default_template = templates.iter()
        .find(|t| t.client_key == "DEFAULT")
        .unwrap_or(if templates.is_empty() { return "[]".to_string(); } else { &templates[0] });

    for r in rows {
        let matched_rule = templates.iter()
            .find(|t| r.client.contains(&t.client_key))
            .unwrap_or(default_template);

        let ttc_num: f64 = r.ttc.parse().unwrap_or(0.0);
        let ht_num: f64 = r.ht.parse().unwrap_or(0.0);
        let tva_num: f64 = r.tva.parse().unwrap_or(0.0);

        let mut context = evalexpr::HashMapContext::new();
        context.set_value("row.ttc".into(), evalexpr::Value::from(ttc_num)).unwrap();
        context.set_value("row.ht".into(), evalexpr::Value::from(ht_num)).unwrap();
        context.set_value("row.tva".into(), evalexpr::Value::from(tva_num)).unwrap();

        let mut computed_lignes = Vec::new();
        for t_row in &matched_rule.rows {
            let amount = evalexpr::eval_with_context(&t_row.formula, &context)
                .unwrap_or(evalexpr::Value::from(0.0)).as_number().unwrap_or(0.0);
            
            computed_lignes.push(serde_json::json!({
                "compte": t_row.compte,
                "type": if t_row.entry_type == crate::ui::EntryType::Debit { "debit" } else { "credit" },
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
        const form = document.getElementById('ecritureForm');
        if (!form) {
            return "NO_FORM";
        }
        
        // Look for signature Axeane structural landmarks
        const hasDateInput = document.querySelector('input[id*="ec-date-creation"]') !== null;
        const hasJournalSelect = document.querySelector('select[id*="jo-eav"]') !== null;
        const hasRowFields = document.querySelector('[id*="cc_0_3"], [id*="exlibelle0"]') !== null;
        
        // If the main form is there along with signature transaction fields, we are on the target page
        if (hasDateInput || hasJournalSelect || hasRowFields) {
            return "READY";
        } else {
            return "WRONG_PAGE";
        }
    })();
    "#.to_string()
}
