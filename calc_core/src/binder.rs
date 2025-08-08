use std::collections::HashSet;

use crate::ast::Expr;

// Very small binder that extracts defined and used symbols for a cell
pub fn bind_cell(expr: &Expr) -> (Vec<String>, Vec<String>) {
    let mut defines = Vec::new();
    let mut uses = HashSet::new();

    match expr {
        Expr::Assign { name, expr } => {
            defines.push(name.clone());
            collect_uses(expr, &mut uses);
        }
        _ => collect_uses(expr, &mut uses),
    }
    (defines, uses.into_iter().collect())
}

fn collect_uses(expr: &Expr, out: &mut HashSet<String>) {
    match expr {
        Expr::Ident(s) => { out.insert(s.clone()); },
        Expr::Unary { expr, .. } => collect_uses(expr, out),
        Expr::Binary { left, right, .. } => { collect_uses(left, out); collect_uses(right, out); },
        Expr::Call { callee, args } => { collect_uses(callee, out); for a in args { collect_uses(a, out); } },
        Expr::Array(items) => { for i in items { collect_uses(i, out); } },
        Expr::Record(fields) => { for (_, e) in fields { collect_uses(e, out); } },
        Expr::Function { params, body, .. } => { collect_uses(body, out); for p in params { out.remove(p); } },
        _ => {}
    }
}

