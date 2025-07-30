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

    // The amb! block must end in an implicit/explicit return
    match block.stmts.split_last() {
        Some((Stmt::Expr(expr, None), rest)) => build_amb(rest.iter(), expr).into(),
        Some((Stmt::Expr(expr, Some(_)), rest)) => {
            match expr {
                Expr::Return(returned_expr) => {
                    if let Some(return_value) = returned_expr.expr.as_ref() {
                        build_amb(rest.iter(), return_value).into()
                    } else {
                        syn::Error::new(expr.span(), 
                            "The amb! block cannot end with en empty return. It must return a value.")
                        .to_compile_error()
                        .into()
                    }
                }
                _ => syn::Error::new(expr.span(), 
                    "The amb! block must end with an expression")
                    .to_compile_error()
                    .into(),
            }
        }
        None => quote!(std::iter::empty::<()>()).into(),
        Some((last_stmt, _)) => syn::Error::new(
            last_stmt.span(),
            "The amb! block must end with an expression",
        )
        .to_compile_error()
        .into(),
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
    let (_, expanded) = build_amb_rec(stmts, final_expr, true);
    expanded
}

/// Recursively desugars `amb!` block into nested iterators and conditions.
fn build_amb_rec<'a, I>(
    mut stmts: I,
    final_expr: &Expr,
    first_call: bool,
) -> (IteratorStage, proc_macro2::TokenStream)
where
    I: Iterator<Item = &'a Stmt> + Clone,
{
    match stmts.next() {
        Some(stmt) => {
            if let Some((pat, iterable)) = extract_choice(stmt) {
                let (iterator_stage, inner) = build_amb_rec(stmts, final_expr, false);
                return match iterator_stage {
                    IteratorStage::InnerMost => (
                        IteratorStage::Outer,
                        if first_call {
                        quote! {
                            (#iterable).into_iter().filter_map(|#pat| {
                                #inner
                            })
                        }

                        } else {
                        quote! {
                            (#iterable).into_iter().filter_map(move |#pat| {
                                #inner
                            })
                        }
                        }
                    ),
                    IteratorStage::Outer => (
                        IteratorStage::Outer,
                        if first_call {
                        quote! {
                            (#iterable).into_iter().flat_map(|#pat| {
                                #inner
                            })
                        }
                        } else {
                        quote! {
                            (#iterable).into_iter().flat_map(move |#pat| {
                                #inner
                            })
                        }
                        }
                    ),
                };
            }

            // Fallback: treat as a normal statement
            let (iterator_stage, inner) = build_amb_rec(stmts, final_expr, first_call);
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

/// Extracts `let <pat> = choice!(<expr>)` pattern. Note that choice! must be used as a top level
/// expression.
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

//macro_rules! require {
//    ($pred:expr) => {
//        if !$pred {
//            return None;
//        }
//    };
//}
