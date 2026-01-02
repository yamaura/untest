use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn untest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(item as ItemFn);

    match generate_test(input) {
        Ok(generated) => generated,
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_test(ast: &ItemFn) -> Result<TokenStream, syn::Error> {
    let ident = &ast.sig.ident;
    let gen_pub = match ast.vis {
        syn::Visibility::Inherited => Some(quote! { pub }),
        _ => None,
    };

    let mod_name = format_ident!("untest_generated_test_{}", ident);
    let wrapper_name = format_ident!("__untest_wrapper_{}", ident);
    let factory_name = format_ident!("__untest_factory_{}", ident);

    let r#gen = quote! {
      #[allow(non_upper_case_globals)]
      #gen_pub const #ident: ::untest::StaticTrial = ::untest::StaticTrial::new(
        stringify!(#ident),
        #wrapper_name,
      );

      fn #wrapper_name() -> ::untest::CaseResult {
        ::untest::IntoCaseResult::into_case_result(#mod_name::#ident())
      }

      fn #factory_name() -> ::untest::Trial {
        ::untest::Trial::Static(&#ident)
      }

      ::untest::inventory::submit! {
        ::untest::TrialFactory(#factory_name)
      }

      mod #mod_name {
        use super::*;
        #gen_pub #ast
      }
    };

    Ok(r#gen.into())
}
