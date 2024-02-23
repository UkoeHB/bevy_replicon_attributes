//module tree
mod inner;

//proc shortcuts
use proc_macro::TokenStream;

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(VisibilityAttribute)]
pub fn derive_visibility_attribute(input: TokenStream) -> TokenStream
{
    inner::derive_visibility_attribute_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------
