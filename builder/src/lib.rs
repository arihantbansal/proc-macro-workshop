use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let name_builder = Ident::new(&format!("{}Builder", name), name.span());

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #name_builder {
                #name_builder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #name_builder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #name_builder {
            pub fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }

            pub fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }

            pub fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }

            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                if self.executable.is_none() {
                    return Err("`executable` is None".into());
                }

                if self.args.is_none() {
                    return Err("`args` is None".into());
                }

                if self.env.is_none() {
                    return Err("`env` is None".into());
                }

                if self.current_dir.is_none() {
                    return Err("`current_dir` is None".into());
                }

                Ok(#name {
                    executable: self.executable.clone().unwrap(),
                    args: self.args.clone().unwrap(),
                    env: self.env.clone().unwrap(),
                    current_dir: self.current_dir.clone().unwrap(),
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
