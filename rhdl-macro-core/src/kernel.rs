use quote::{quote, ToTokens};
use syn::spanned::Spanned;
type TS = proc_macro2::TokenStream;
type Result<T> = syn::Result<T>;

pub fn hdl_block(block: &syn::Block) -> Result<TS> {
    let stmts = block.stmts.iter().map(stmt).collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        vec![#(#stmts),*]
    })
}

fn stmt(statement: &syn::Stmt) -> Result<TS> {
    match statement {
        syn::Stmt::Local(local) => stmt_local(local),
        syn::Stmt::Expr(expr, semi) => {
            let expr = hdl_expr(expr)?;
            if semi.is_some() {
                Ok(quote! {
                    rhdl_core::ast::Stmt::Semi(#expr)
                })
            } else {
                Ok(quote! {
                    rhdl_core::ast::Stmt::Expr(#expr)
                })
            }
        }
        _ => Err(syn::Error::new(
            statement.span(),
            "Unsupported statement type",
        )),
    }
}

fn stmt_local(local: &syn::Local) -> Result<TS> {
    let pattern = hdl_pat(&local.pat)?;
    let local_init = local
        .init
        .as_ref()
        .map(|x| hdl_expr(&x.expr))
        .ok_or(syn::Error::new(
            local.span(),
            "Unsupported local declaration",
        ))??;
    Ok(quote! {
        rhdl_core::ast::Stmt::Local(Local{pattern: #pattern, value: Box::new(#local_init)})
    })
}

fn hdl_pat(pat: &syn::Pat) -> Result<TS> {
    match pat {
        syn::Pat::Ident(ident) => {
            let name = &ident.ident;
            let mutability = ident.mutability.is_some();
            if ident.by_ref.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "Unsupported reference pattern",
                ));
            }
            Ok(quote! {
                rhdl_core::ast::LocalPattern::Ident(
                    rhdl_core::ast::LocalIdent{
                        name: stringify!(#name).to_string(),
                        mutable: #mutability
                    }
                )
            })
        }
        syn::Pat::TupleStruct(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(hdl_pat)
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                rhdl_core::ast::LocalPattern::Tuple(vec![#(#elems),*])
            })
        }
        _ => Err(syn::Error::new(pat.span(), "Unsupported pattern type")),
    }
}

fn hdl_expr(expr: &syn::Expr) -> Result<TS> {
    match expr {
        syn::Expr::Lit(expr) => hdl_lit(expr),
        syn::Expr::Binary(expr) => hdl_binary(expr),
        syn::Expr::Unary(expr) => hdl_unary(expr),
        syn::Expr::Group(expr) => hdl_group(expr),
        syn::Expr::Paren(expr) => hdl_paren(expr),
        syn::Expr::Assign(expr) => hdl_assign(expr),
        syn::Expr::Path(expr) => hdl_path(&expr.path),
        syn::Expr::Struct(expr) => hdl_struct(expr),
        syn::Expr::Block(expr) => hdl_block(&expr.block),
        syn::Expr::Field(expr) => hdl_field_expression(expr),
        syn::Expr::If(expr) => hdl_if(expr),
        syn::Expr::Let(expr) => hdl_let(expr),
        syn::Expr::Match(expr) => hdl_match(expr),
        syn::Expr::Range(expr) => hdl_range(expr),
        syn::Expr::Try(expr) => hdl_try(expr),
        syn::Expr::Return(expr) => hdl_return(expr),
        syn::Expr::Tuple(expr) => hdl_tuple(expr),
        syn::Expr::Repeat(expr) => hdl_repeat(expr),
        syn::Expr::ForLoop(expr) => hdl_for_loop(expr),
        syn::Expr::While(expr) => hdl_while_loop(expr),
        _ => Err(syn::Error::new(
            expr.span(),
            format!("Unsupported expression type {}", quote!(#expr)),
        )),
    }
}

fn hdl_for_loop(expr: &syn::ExprForLoop) -> Result<TS> {
    let pat = hdl_pat(&expr.pat)?;
    let body = hdl_block(&expr.body)?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::ForLoop(
            rhdl_core::ast::ExprForLoop {
                pat: #pat,
                expr: Box::new(#expr),
                body: #body,
            }
        )
    })
}

fn hdl_while_loop(expr: &syn::ExprWhile) -> Result<TS> {
    let cond = hdl_expr(&expr.cond)?;
    let body = hdl_block(&expr.body)?;
    Ok(quote! {
        rhdl_core::ast::Expr::While(
            rhdl_core::ast::ExprWhile {
                cond: Box::new(#cond),
                body: #body,
            }
        )
    })
}

fn hdl_repeat(expr: &syn::ExprRepeat) -> Result<TS> {
    let len = hdl_expr(&expr.len)?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Repeat(
            rhdl_core::ast::ExprRepeat {
                expr: Box::new(#expr),
                len: Box::new(#len),
            }
        )
    })
}

fn hdl_tuple(expr: &syn::ExprTuple) -> Result<TS> {
    let elems = expr
        .elems
        .iter()
        .map(hdl_expr)
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast::Expr::Tuple(vec![#(#elems),*])
    })
}

fn hdl_group(expr: &syn::ExprGroup) -> Result<TS> {
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Group(Box::new(#expr))
    })
}

fn hdl_paren(expr: &syn::ExprParen) -> Result<TS> {
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Paren(Box::new(#expr))
    })
}

fn hdl_return(expr: &syn::ExprReturn) -> Result<TS> {
    let expr = expr
        .expr
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(Box::new(#x))})
        .unwrap_or_else(|| quote! {None});
    Ok(quote! {
        rhdl_core::ast::Expr::Return(#expr)
    })
}

fn hdl_try(expr: &syn::ExprTry) -> Result<TS> {
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Try(Box::new(#expr))
    })
}

fn hdl_range(expr: &syn::ExprRange) -> Result<TS> {
    let start = expr
        .start
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(Box::new(#x))})
        .unwrap_or_else(|| quote! {None});
    let end = expr
        .end
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(Box::new(#x))})
        .unwrap_or_else(|| quote! {None});
    let limits = match expr.limits {
        syn::RangeLimits::HalfOpen(_) => quote!(rhdl_core::ast::RangeLimits::HalfOpen),
        syn::RangeLimits::Closed(_) => quote!(rhdl_core::ast::RangeLimits::Closed),
    };
    Ok(quote! {
        rhdl_core::ast::Expr::Range(
            rhdl_core::ast::ExprRange {
                start: #start,
                end: #end,
                limits: #limits,
            }
        )
    })
}

fn hdl_match(expr: &syn::ExprMatch) -> Result<TS> {
    let arms = expr.arms.iter().map(hdl_arm).collect::<Result<Vec<_>>>()?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Match(
            rhdl_core::ast::ExprMatch {
                expr: Box::new(#expr),
                arms: vec![#(#arms),*],
            }
        )
    })
}

fn hdl_arm(arm: &syn::Arm) -> Result<TS> {
    let pat = hdl_pat(&arm.pat)?;
    let guard = arm.guard.as_ref().map(|(_if, x)| hdl_expr(x)).transpose()?;
    let body = hdl_expr(&arm.body)?;
    Ok(quote! {
        rhdl_core::ast::Arm {
            pattern: #pat,
            guard: #guard,
            body: Box::new(#body),
        }
    })
}

fn hdl_let(expr: &syn::ExprLet) -> Result<TS> {
    let pattern = hdl_pat(&expr.pat)?;
    let value = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Let(
            rhdl_core::ast::ExprLet {
                pattern: #pattern,
                value: Box::new(#value),
            }
        )
    })
}

fn hdl_if(expr: &syn::ExprIf) -> Result<TS> {
    let cond = hdl_expr(&expr.cond)?;
    let then = hdl_block(&expr.then_branch)?;
    let else_ = expr
        .else_branch
        .as_ref()
        .map(|x| hdl_expr(&x.1))
        .transpose()?;
    Ok(quote! {
        rhdl_core::ast::Expr::If(
            rhdl_core::ast::ExprIf {
                cond: Box::new(#cond),
                then_branch: #then,
                else_branch: #else_,
            }
        )
    })
}

fn hdl_struct(structure: &syn::ExprStruct) -> Result<TS> {
    let path = hdl_path(&structure.path)?;
    let fields = structure
        .fields
        .iter()
        .map(hdl_field)
        .collect::<Result<Vec<_>>>()?;
    if structure.qself.is_some() {
        return Err(syn::Error::new(
            structure.span(),
            "Unsupported qualified self",
        ));
    }
    let rest = structure.rest.as_ref().map(|x| hdl_expr(&x)).transpose()?;
    Ok(quote! {
        rhdl_core::ast::Expr::Struct(
            rhdl_core::ast::ExprStruct {
                path: #path,
                fields: vec![#(#fields),*],
                rest: #rest,
            }
        )
    })
}

fn hdl_path(path: &syn::Path) -> Result<TS> {
    let ident = path
        .get_ident()
        .ok_or(syn::Error::new(path.span(), "Unsupported path expression"))?;
    Ok(quote! {
        rhdl_core::ast::Expr::Ident(stringify!(#ident).to_string())
    })
}

fn hdl_assign(assign: &syn::ExprAssign) -> Result<TS> {
    let left = hdl_expr(&assign.left)?;
    let right = hdl_expr(&assign.right)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Assign(Box::new(#left), Box::new(#right))
    })
}

fn hdl_field_expression(field: &syn::ExprField) -> Result<TS> {
    let expr = hdl_expr(&field.base)?;
    let member = hdl_member(&field.member)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Field(
            rhdl_core::ast::ExprField {
                expr: Box::new(#expr),
                member: #member,
            }
        )
    })
}

fn hdl_field(field: &syn::FieldValue) -> Result<TS> {
    let member = hdl_member(&field.member)?;
    let expr = hdl_expr(&field.expr)?;
    Ok(quote! {
        rhdl_core::ast::ExprField {
            member: #member,
            expr: Box::new(#expr),
        }
    })
}

fn hdl_member(member: &syn::Member) -> Result<TS> {
    Ok(match member {
        syn::Member::Named(ident) => quote! {
            rhdl_core::ast::Member::Named(stringify!(#ident).to_string())
        },
        syn::Member::Unnamed(index) => {
            let index = index.index;
            quote! {
                rhdl_core::ast::Member::Unnamed(#index)
            }
        }
    })
}

fn hdl_unary(unary: &syn::ExprUnary) -> Result<TS> {
    let op = match unary.op {
        syn::UnOp::Neg(_) => quote!(rhdl_core::ast::UnOp::Neg),
        syn::UnOp::Not(_) => quote!(rhdl_core::ast::UnOp::Not),
        _ => return Err(syn::Error::new(unary.span(), "Unsupported unary operator")),
    };
    let expr = hdl_expr(&unary.expr)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Unary(
            rhdl_core::ast::ExprUnary
            {
                op: #op,
                expr: Box::new(#expr)
            }
        )
    })
}

fn hdl_binary(binary: &syn::ExprBinary) -> Result<TS> {
    let op = match binary.op {
        syn::BinOp::Add(_) => quote!(rhdl_core::ast::BinOp::Add),
        syn::BinOp::Sub(_) => quote!(rhdl_core::ast::BinOp::Sub),
        syn::BinOp::Mul(_) => quote!(rhdl_core::ast::BinOp::Mul),
        syn::BinOp::And(_) => quote!(rhdl_core::ast::BinOp::And),
        syn::BinOp::Or(_) => quote!(rhdl_core::ast::BinOp::Or),
        syn::BinOp::BitXor(_) => quote!(rhdl_core::ast::BinOp::BitXor),
        syn::BinOp::BitAnd(_) => quote!(rhdl_core::ast::BinOp::BitAnd),
        syn::BinOp::BitOr(_) => quote!(rhdl_core::ast::BinOp::BitOr),
        syn::BinOp::Shl(_) => quote!(rhdl_core::ast::BinOp::Shl),
        syn::BinOp::Shr(_) => quote!(rhdl_core::ast::BinOp::Shr),
        syn::BinOp::Eq(_) => quote!(rhdl_core::ast::BinOp::Eq),
        syn::BinOp::Lt(_) => quote!(rhdl_core::ast::BinOp::Lt),
        syn::BinOp::Le(_) => quote!(rhdl_core::ast::BinOp::Le),
        syn::BinOp::Ne(_) => quote!(rhdl_core::ast::BinOp::Ne),
        syn::BinOp::Ge(_) => quote!(rhdl_core::ast::BinOp::Ge),
        syn::BinOp::Gt(_) => quote!(rhdl_core::ast::BinOp::Gt),
        syn::BinOp::AddAssign(_) => quote!(rhdl_core::ast::BinOp::AddAssign),
        syn::BinOp::SubAssign(_) => quote!(rhdl_core::ast::BinOp::SubAssign),
        syn::BinOp::MulAssign(_) => quote!(rhdl_core::ast::BinOp::MulAssign),
        syn::BinOp::BitXorAssign(_) => quote!(rhdl_core::ast::BinOp::BitXorAssign),
        syn::BinOp::BitAndAssign(_) => quote!(rhdl_core::ast::BinOp::BitAndAssign),
        syn::BinOp::BitOrAssign(_) => quote!(rhdl_core::ast::BinOp::BitOrAssign),
        syn::BinOp::ShlAssign(_) => quote!(rhdl_core::ast::BinOp::ShlAssign),
        syn::BinOp::ShrAssign(_) => quote!(rhdl_core::ast::BinOp::ShrAssign),
        _ => {
            return Err(syn::Error::new(
                binary.span(),
                "Unsupported binary operator",
            ))
        }
    };
    let left = hdl_expr(&binary.left)?;
    let right = hdl_expr(&binary.right)?;
    Ok(quote! {
        rhdl_core::ast::Expr::Binary(
            rhdl_core::ast::ExprBinary {
                op: #op,
                lhs: Box::new(#left),
                rhs: Box::new(#right),
            }
        )
    })
}

fn hdl_lit(lit: &syn::ExprLit) -> Result<TS> {
    let lit = &lit.lit;
    match lit {
        syn::Lit::Int(int) => {
            let value = int.token();
            Ok(quote! {
                rhdl_core::ast::Expr::Lit(
                    rhdl_core::ast::ExprLit::Int(stringify!(#value).to_string())
                )
            })
        }
        syn::Lit::Bool(boolean) => {
            let value = boolean.value;
            Ok(quote! {
                rhdl_core::ast::Expr::Lit(
                    rhdl_core::ast::ExprLit::Bool(#value)
                )
            })
        }
        _ => Err(syn::Error::new(lit.span(), "Unsupported literal type")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_block() {
        let test_code = quote! {
            {
                let a = 1;
                let b = 2;
                let q = 0x1234_u32;
                let c = a + b;
                let mut d = 3;
                let g = Foo {r: 1, g: 120, b: 33};
                let h = g.r;
                c
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
    #[test]
    fn test_precedence_parser() {
        let test_code = quote! {
            {
                1 + 3 * 9
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
}