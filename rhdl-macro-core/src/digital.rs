use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

use crate::digital_enum::derive_digital_enum;

pub fn derive_digital(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_digital_struct(decl),
        Data::Enum(_e) => derive_digital_enum(decl),
        _ => bail!("Only structs and enums can be digital"),
        //        Data::Union(_u) => derive_digital_union(decl),
    }
}

fn derive_digital_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_digital_named_struct(decl),
            syn::Fields::Unnamed(_) => derive_digital_tuple_struct(decl),
            syn::Fields::Unit => Err(anyhow!("Unit structs are not digital")),
        },
        _ => Err(anyhow!("Only structs can be digital")),
    }
}

fn derive_digital_tuple_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .enumerate()
                .map(|(ndx, field)| syn::LitInt::new(&ndx.to_string(), field.span()));
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #struct_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                        #(
                            <#field_types as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                        #(
                            self.#fields2.record(tag, &mut logger);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be digital")),
    }
}

fn derive_digital_named_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|field| &field.ident);
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #struct_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                        #(
                            <#field_types as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                        #(
                            self.#fields2.record(tag, &mut logger);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be digital")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::assert_tokens_eq;

    #[test]
    fn test_digital_enum() {
        let decl = quote!(
            pub enum State {
                Init,
                Boot,
                Running,
                Stop,
                Boom,
            }
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for State {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::Enum {
                        variants: vec![
                            rhdl_core::Variant {
                                name: "Init".to_string(),
                            },
                            rhdl_core::Variant {
                                name: "Boot".to_string(),
                            },
                            rhdl_core::Variant {
                                name: "Running".to_string(),
                            },
                            rhdl_core::Variant {
                                name: "Stop".to_string(),
                            },
                            rhdl_core::Variant {
                                name: "Boom".to_string(),
                            },
                        ],
                    }
                }
                fn bin(&self) -> Vec<bool> {
                    match self {
                        Self::Init => rhdl_bits::Bits::<3>::(0).to_bools(),
                        Self::Boot => rhdl_bits::Bits::<3>::(1).to_bools(),
                        Self::Running => rhdl_bits::Bits::<3>::(2).to_bools(),
                        Self::Stop => rhdl_bits::Bits::<3>::(3).to_bools(),
                        Self::Boom => rhdl_bits::Bits::<3>::(4).to_bools(),
                    }
                }
                fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                    builder.allocate(tag, 0);
                }
                fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                    match self {
                        Self::Init => logger.write_string(tag, stringify!(Init)),
                        Self::Boot => logger.write_string(tag, stringify!(Boot)),
                        Self::Running => logger.write_string(tag, stringify!(Running)),
                        Self::Stop => logger.write_string(tag, stringify!(Stop)),
                        Self::Boom => logger.write_string(tag, stringify!(Boom)),
                    }
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_digital_proc_macro() {
        let decl = quote!(
            pub struct NestedBits {
                nest_1: bool,
                nest_2: u8,
                nest_3: TwoBits,
            }
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for NestedBits {
                fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(nest_1)));
                    <u8 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(nest_2)));
                    <TwoBits as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(nest_3)));
                }
                fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                    self.nest_1.record(tag, &mut logger);
                    self.nest_2.record(tag, &mut logger);
                    self.nest_3.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_digital_with_struct() {
        let decl = quote!(
            pub struct Inputs {
                pub input: u32,
                pub write: bool,
                pub read: bool,
            }
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::Struct {
                        fields: vec![
                            rhdl_core::Field {
                                name: "input".to_string(),
                                kind: <u32 as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "write".to_string(),
                                kind: <bool as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "read".to_string(),
                                kind: <bool as rhdl_core::Digital>::static_kind(),
                            },
                        ],
                    }
                }
                fn bin(self) -> Vec<bool> {
                    let mut v = self.input.bin();
                    v.extend(self.write.bin());
                    v.extend(self.read.bin());
                    v
                }
                fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(input)));
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(write)));
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(read)));
                }

                fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                    self.input.record(tag, &mut logger);
                    self.write.record(tag, &mut logger);
                    self.read.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_struct_with_tuple_field() {
        let decl = quote!(
            pub struct Inputs {
                pub input: u32,
                pub write: bool,
                pub read: (bool, bool),
            }
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::Struct {
                        fields: vec![
                            rhdl_core::Field {
                                name: "input".to_string(),
                                kind: <u32 as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "write".to_string(),
                                kind: <bool as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "read".to_string(),
                                kind: <(bool, bool) as rhdl_core::Digital>::static_kind(),
                            },
                        ],
                    }
                }
                fn bin(self) -> Vec<bool> {
                    let mut v = self.input.bin();
                    v.extend(self.write.bin());
                    v.extend(self.read.bin());
                    v
                }
                fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(input)));
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(write)));
                    <(bool, bool) as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(read)));
                }

                fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                    self.input.record(tag, &mut logger);
                    self.write.record(tag, &mut logger);
                    self.read.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_digital_with_tuple_struct() {
        let decl = quote!(
            pub struct Inputs(pub u32, pub bool, pub bool);
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::Struct {
                        fields: vec![
                            rhdl_core::Field {
                                name: "0".to_string(),
                                kind: <u32 as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "1".to_string(),
                                kind: <bool as rhdl_core::Digital>::static_kind(),
                            },
                            rhdl_core::Field {
                                name: "2".to_string(),
                                kind: <bool as rhdl_core::Digital>::static_kind(),
                            },
                        ],
                    }
                }
                fn bin() -> Vec<bool> {
                    let mut v = self.0.bin();
                    v.extend(self.1.bin());
                    v.extend(self.2.bin());
                    v
                }
                fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(0)));
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(1)));
                    <bool as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(2)));
                }

                fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                    self.0.record(tag, &mut logger);
                    self.1.record(tag, &mut logger);
                    self.2.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }
}