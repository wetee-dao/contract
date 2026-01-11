use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident};

/// 使用 quote! 一次性生成 subnet_contract, cloud_contract, cloud_call_builder
#[proc_macro]
pub fn setup_contracts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SetupContractsInput);

    let client = input.client;

    let expanded = quote! {
        let pod_code = #client
            .upload("pod", &ink_e2e::alice())
            .submit()
            .await
            .expect("pod upload failed");

        let subnet_contract = #client
            .instantiate("subnet", &ink_e2e::alice(), &mut subnet::SubnetRef::new())
            .submit()
            .await
            .expect("subnet upload failed");

        let cloud_contract = #client
            .instantiate(
                "cloud",
                &ink_e2e::alice(),
                &mut CloudRef::new(subnet_contract.addr, pod_code.code_hash),
            )
            .submit()
            .await
            .expect("cloud init failed");

        let mut cloud_call_builder = cloud_contract.call_builder::<Cloud>();
    };

    TokenStream::from(expanded)
}

struct SetupContractsInput {
    client: Ident,
}

impl syn::parse::Parse for SetupContractsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let client: Ident = input.parse()?;
        Ok(SetupContractsInput { client })
    }
}
