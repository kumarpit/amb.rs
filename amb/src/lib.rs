use quote::quote;
use syn::{Block, Expr, Stmt, StmtMacro, spanned::Spanned};

#[proc_macro]
pub fn amb(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let block = syn::parse_macro_input!(input as Block);
    if let Some((last, stmts)) = block.stmts.split_last() {
        match last {
            // The last statement in the amb block must evaluate to a value
            Stmt::Expr(expr, None) => parse_amb_block(stmts.iter(), expr, true).into(),
            _ => syn::Error::new(last.span(), "The amb! block must end with an expression")
                .to_compile_error()
                .into(),
        }
    } else {
        // An empty block produces an empty iterator
        quote!(std::iter::empty::<()>()).into()
    }
}

fn parse_amb_block<'a, I>(
    mut stmts: I,
    final_expr: &Expr,
    is_first_iterator: bool,
) -> proc_macro2::TokenStream
where
    I: Iterator<Item = &'a Stmt> + Clone,
{
    if let Some(stmt) = stmts.next() {
        let mut pat = None;
        let mut iterable = None;

        // Case 1: `let <pat> = choice!(<iterable>);`
        if let Stmt::Local(local) = stmt {
            // Matches the part including and after the =
            if let Some(init) = &local.init {
                // The iterable must be wrapped in the `choice!` macro
                if let Expr::Macro(expr_macro) = &*init.expr {
                    if expr_macro.mac.path.is_ident("choice") {
                        pat = Some(&local.pat);
                        iterable = Some(&expr_macro.mac.tokens);
                    }
                }
            }
        }

        let is_case_1 = pat.is_some();
        let inner_iterator = parse_amb_block(
            stmts,
            final_expr,
            if is_first_iterator && is_case_1 {
                false
            } else {
                true
            },
        );

        if is_case_1 {
            if is_first_iterator {
                return quote! {
                    (#iterable).into_iter().flat_map(move |#pat| {
                        #inner_iterator
                    })
                };
            } else {
                return quote! {
                    (#iterable).into_iter().filter_map(move |#pat| {
                        #inner_iterator
                    })
                };
            }
        }

        // Case 2: `require!(<pred>)`
        if let Stmt::Macro(StmtMacro { mac, .. }) = stmt {
            if mac.path.is_ident("require") {
                if let Ok(pred) = mac.parse_body::<Expr>() {
                    return quote! {
                        if #pred {
                            #inner_iterator
                        } else {
                            None
                        }
                    };
                }
            }
        }

        // Case 3: Any other statement is simply prepended.
        return quote!({
            #stmt
            #inner_iterator
        });
    } else {
        // Base case
        quote!(Some(#final_expr))
    }
}
