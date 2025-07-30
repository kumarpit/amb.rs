use quote::quote;
use syn::{Block, Expr, Stmt, StmtMacro, spanned::Spanned};

/// The `amb!` macro enables backtracking search over a space of choices.
///
/// It supports three types of statements:
/// - `let <pat> = choice!(<iter>)` — introduces a non-deterministic choice.
/// - `require!(<cond>)` — prunes execution paths that don't satisfy the predicate.
/// - Any other statement is executed normally.
///
/// The block must end in a pure expression (not a semicolon-terminated statement).
#[proc_macro]
pub fn amb(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let block = syn::parse_macro_input!(input as Block);

    match block.stmts.split_last() {
        Some((Stmt::Expr(expr, None), rest)) => build_amb(rest.iter(), expr).into(),
        Some((last_stmt, _)) => syn::Error::new(
            last_stmt.span(),
            "The amb! block must end with an expression",
        )
        .to_compile_error()
        .into(),
        None => quote!(std::iter::empty::<()>()).into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IteratorStage {
    /// The innermost iterator must be combined with filter_map
    InnerMost,
    /// All outer iterators are combined with flat_map
    Outer,
}

fn build_amb<'a, I>(stmts: I, final_expr: &Expr) -> proc_macro2::TokenStream
where
    I: Iterator<Item = &'a Stmt> + Clone,
{
    let (_, expanded) = build_amb_rec(stmts, final_expr);
    expanded
}

/// Recursively desugars `amb!` block into nested iterators and conditions.
fn build_amb_rec<'a, I>(
    mut stmts: I,
    final_expr: &Expr,
) -> (IteratorStage, proc_macro2::TokenStream)
where
    I: Iterator<Item = &'a Stmt> + Clone,
{
    match stmts.next() {
        Some(stmt) => {
            if let Some((pat, iterable)) = extract_choice(stmt) {
                let (iterator_stage, inner) = build_amb_rec(stmts, final_expr);
                return match iterator_stage {
                    IteratorStage::InnerMost => (
                        IteratorStage::Outer,
                        quote! {
                            (#iterable).into_iter().filter_map(move |#pat| {
                                #inner
                            })
                        },
                    ),
                    IteratorStage::Outer => (
                        IteratorStage::Outer,
                        quote! {
                            (#iterable).into_iter().flat_map(move |#pat| {
                                #inner
                            })
                        },
                    ),
                };
            }

            if let Some(pred) = extract_require(stmt) {
                let (iterator_stage, inner) = build_amb_rec(stmts, final_expr);
                return (
                    iterator_stage,
                    quote! {
                        if #pred {
                            #inner
                        } else {
                            None
                        }
                    },
                );
            }

            // Fallback: treat as a normal statement
            let (iterator_stage, inner) = build_amb_rec(stmts, final_expr);
            (
                iterator_stage,
                quote!({
                    #stmt
                    #inner
                }),
            )
        }

        None => (IteratorStage::InnerMost, quote!(Some(#final_expr))),
    }
}

/// Extracts `let <pat> = choice!(<expr>)` pattern.
fn extract_choice(stmt: &Stmt) -> Option<(&syn::Pat, &proc_macro2::TokenStream)> {
    match stmt {
        Stmt::Local(local) => {
            let init_expr = local.init.as_ref()?.expr.as_ref();
            if let Expr::Macro(mac_expr) = init_expr {
                if mac_expr.mac.path.is_ident("choice") {
                    return Some((&local.pat, &mac_expr.mac.tokens));
                }
            }
            None
        }
        _ => None,
    }
}

/// Extracts `require!(<predicate>)` pattern.
fn extract_require(stmt: &Stmt) -> Option<Expr> {
    match stmt {
        Stmt::Macro(StmtMacro { mac, .. }) if mac.path.is_ident("require") => mac.parse_body().ok(),
        _ => None,
    }
}
