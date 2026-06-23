use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(HotReload, attributes(nested))]
pub fn derive_hot_reload(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let named_fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("#[derive(HotReload)] only works on named-field structs."),
        },
        _ => panic!("#[derive(HotReload)] only works on structs."),
    };

    let field_apply_blocks = named_fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("named field");
        let field_name_str = field_ident.to_string();
        let field_ty = &field.ty;

        let is_nested = field.attrs.iter().any(|attr| attr.path().is_ident("nested"));

        if is_nested {
            quote! {
                if let Some(__raw) = __table.get(#field_name_str) {
                    if let Some(__sub_table) = __raw.as_table() {
                        self.#field_ident.apply_toml(__sub_table);
                    } else {
                        ::log::warn!(
                            "[snapcfg] HotReload: expected '{}' to be a TOML table.",
                            #field_name_str
                        );
                    }
                }
            }
        } else {
            let warn_msg = format!(
                "[snapcfg] HotReload: type mismatch for field '{}' \
                 (expected: {}). Kept the old value. Error: {{}}",
                field_name_str,
                type_display(field_ty),
            );


            quote! {
                if let Some(__raw) = __table.get(#field_name_str) {
                    match ::toml::Value::try_into::<#field_ty>(__raw.clone()) {
                        Ok(__v)  => { self.#field_ident = __v; }
                        Err(__e) => { ::log::warn!(#warn_msg, __e); }
                    }
                }
            }
        }
    });

    let expanded = quote! {
        const _: () = {
            #[allow(unused_imports)]
            use ::snapcfg::__private::HotReloadable as __HotReloadable;

            impl __HotReloadable for #struct_name {
                fn apply_toml(&mut self, __table: &::toml::Table) {
                    #(#field_apply_blocks)*
                }
            }
        };
    };

    expanded.into()
}

fn type_display(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => "?".to_string(),
    }
}
