use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = &input.data;

    let name_builder = Ident::new(&format!("{}Builder", name), name.span());

    let mut fields_map: HashMap<String, String> = HashMap::new();
    let mut fields: Vec<proc_macro2::TokenStream> = vec![];
    let mut field_builder: Vec<proc_macro2::TokenStream> = vec![];
    let mut empty_fields: Vec<proc_macro2::TokenStream> = vec![];
    let mut field_fns: Vec<proc_macro2::TokenStream> = vec![];
    let mut field_check_none: Vec<proc_macro2::TokenStream> = vec![];

    match data {
        Data::Struct(s) => {
            if let Fields::Named(named_fields) = &s.fields {
                for named_field in named_fields.named.iter() {
                    if let Some(field_name) = &named_field.ident {
                        let field_type = &named_field.ty;
                        if let Type::Path(path) = field_type {
                            let segments = &path.path.segments;

                            fields_map.insert(
                                field_name.to_string(),
                                path.to_token_stream().to_string().replace(" ", ""),
                            );

                            empty_fields.push(quote!(
                                #field_name: None
                            ));

                            if !segments.is_empty() && segments[0].ident.to_string().eq("Option") {
                                field_builder.push(quote!(
                                    #field_name: self.#field_name.clone()
                                ));

                                if let Some(og_type) = segments[0]
                                    .arguments
                                    .to_token_stream()
                                    .to_string()
                                    .split(' ')
                                    .find(|el| !["<", ">", "(", ")"].contains(el))
                                {
                                    let og_type_ident = Ident::new(og_type, field_name.span());
                                    fields.push(quote!(
                                        #field_name: std::option::Option::<#og_type_ident>
                                    ));

                                    field_fns.push(quote!(
                                        pub fn #field_name(&mut self, #field_name: #og_type_ident) -> &mut Self {
                                            self.#field_name = Some(#field_name);
                                            self
                                        }
                                    ));
                                } else {
                                    panic!("Could not find generic type for optional param");
                                }
                            } else {
                                fields.push(quote!(
                                    #field_name: std::option::Option<#field_type>
                                ));

                                field_builder.push(quote!(
                                    #field_name: self.#field_name.clone().ok_or(concat!(stringify!(#field_name), " is not set"))?
                                ));

                                field_fns.push(quote!(
                                    pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                                        self.#field_name = Some(#field_name);
                                        self
                                    }
                                ));

                                field_check_none.push(quote!(
                                    if self.#field_name.is_none() {
                                        return Err(format!("`{}` is not set", stringify!(#field_name)).into())
                                    }
                                ));
                            }
                        }
                    }
                }
            }
        }
        _ => panic!("Unexpected data type"),
    }

    let expanded = quote! {
        pub struct #name_builder {
            #( #fields ),*
        }

        impl #name {
            pub fn builder() -> #name_builder {
                #name_builder {
                    #( #empty_fields ),*
                }
            }
        }

        impl #name_builder {
            #( #field_fns )*

            pub fn build(&mut self) -> Result<#name, std::boxed::Box<dyn std::error::Error>> {
                #( #field_check_none )*

                Ok(#name {
                    #( #field_builder ),*
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
