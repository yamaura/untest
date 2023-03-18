use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let gen = quote! {
        extern crate test;
        #[allow(non_upper_case_globals)]
        pub const #ident: test::TestDescAndFn = test::TestDescAndFn {
          desc: test::TestDesc {
              name: test::StaticTestName(stringify!(#ident)),
              ignore: false,
              ignore_message: ::core::option::Option::None,
              compile_fail: false,
              no_run: false,
              should_panic: test::ShouldPanic::No,
              test_type: test::TestType::IntegrationTest,
          },
          testfn: test::StaticTestFn(|| test::assert_test_result(untest_generated_test::#ident())),
      };
      mod untest_generated_test {
        #gen_pub #ast
      }
    };

    Ok(gen.into())
}
