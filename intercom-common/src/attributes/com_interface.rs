
use prelude::*;
use super::common::*;

use std::iter;

use idents;
use utils;
use tyhandlers::{Direction, TypeSystem};
use model;
use methodinfo::ComMethodInfo;

extern crate proc_macro;
use syn::*;

/// Expands the `com_interface` attribute.
///
/// The attribute expansion results in the following items:
///
/// - Global IID for the interface.
/// - Virtual table struct definition for the interface.
/// - Implementation for the delegating methods when calling the COM interface
///   from Rust.
pub fn expand_com_interface(
    attr_tokens: TokenStreamNightly,
    item_tokens: TokenStreamNightly,
) -> Result<TokenStreamNightly, model::ParseError>
{
    // Parse the attribute.
    let mut output = vec![];
    let itf = model::ComInterface::parse(
            &lib_name(),
            attr_tokens.into(),
            &item_tokens.to_string() )?;
    let itf_ident = itf.name();

    // Impls for Rust-to-COM calls using Automation type system.
    for ( &ts, itf_variant ) in itf.variants() {
        process_itf_variant( &itf, ts, itf_variant, &mut output );
    }

    // If this is a trait based interface, implement Deref for ComItf.
    //
    // Trait based interfaces can always be Deref'd from ComItf into &Trait.
    // Struct based interfaces can only be Deref'd if the struct has the
    // interface in its vtable list. This is decided in the com_class
    // attribute.
    if itf.item_type() == utils::InterfaceType::Trait {
        output.push( quote!(
            impl ::std::ops::Deref for ::intercom::ComItf< #itf_ident > {
                type Target = #itf_ident;
                fn deref( &self ) -> &Self::Target {
                    self
                }
            }
        ) );
    }

    Ok( tokens_to_tokenstream( item_tokens, output ) )
}

/// Processes the interface type system variant.
///
/// # Arguments
///
/// * `itf` - Interface details.
/// * `ts` - Type system the variant represents.
/// * `itf_variant` - Interface variant details.
/// * `output` - Output emitted by the attribute macro.
fn process_itf_variant(
    itf : &model::ComInterface,
    ts : TypeSystem,
    itf_variant : &model::ComInterfaceVariant,
    output : &mut Vec<TokenStream>,
) {

    let itf_ident = itf.name();
    let visibility = itf.visibility();
    let iid_ident = idents::iid( itf_variant.unique_name() );
    let vtable_ident = idents::vtable_struct( itf_variant.unique_name() );

    // IID_IInterface GUID.
    let iid_tokens = utils::get_guid_tokens( itf_variant.iid() );
    let iid_doc = format!( "`{}` interface ID.", itf_ident );
    output.push( quote!(
        #[doc = #iid_doc]
        #[allow(non_upper_case_globals)]
        #visibility const #iid_ident : ::intercom::IID = #iid_tokens;
    ) );

    // Create a vector for the virtual table fields and insert the base
    // interface virtual table in it if required.
    let mut vtbl_fields = vec![];
    if let Some( ref base ) = *itf.base_interface() {
        let vtbl = match base.to_string().as_ref() {
            "IUnknown" => quote!( ::intercom::IUnknownVtbl ),
            _ => { let vtbl = idents::vtable_struct( &base ); quote!( #vtbl ) }
        };
        vtbl_fields.push( quote!( pub __base : #vtbl ) );
    }

    // Gather all the trait methods for the remaining vtable fields.
    let calling_convention = get_calling_convetion();
    for method_info in itf_variant.methods() {

        let method_ident = &method_info.unique_name;
        let in_out_args = method_info.raw_com_args()
                .into_iter()
                .map( |com_arg| {
                    let name = &com_arg.name;
                    let com_ty = &com_arg.handler.com_ty();
                    let dir = match com_arg.dir {
                        Direction::In => quote!(),
                        Direction::Out | Direction::Retval => quote!( *mut )
                    };
                    quote!( #name : #dir #com_ty )
                } );
        let self_arg = quote!( self_vtable : ::intercom::RawComPtr );
        let args = iter::once( self_arg ).chain( in_out_args );

        // Create the vtable field and add it to the vector of fields.
        let ret_ty = method_info.returnhandler.com_ty();
        vtbl_fields.push( quote!(
            pub #method_ident :
                unsafe extern #calling_convention fn( #( #args ),* ) -> #ret_ty
        ) );
    }

    // Create the vtable. We've already gathered all the vtable method
    // pointer fields so defining the struct is simple enough.
    output.push( quote!(
        #[allow(non_camel_case_types)]
        #[repr(C)]
        #[doc(hidden)]
        #visibility struct #vtable_ident { #( #vtbl_fields, )* }
    ) );

    // COM delegation implementation.
    // This is done only for the primary interface, whic is currently always the Automation
    // interface.
    if ts == TypeSystem::Automation {

        // IidOf trait impl.
        let iidof_doc = format!( "Returns `{}`.", iid_ident );
        output.push( quote!(
            impl ::intercom::IidOf for #itf_ident {
                #[doc = #iidof_doc]
                fn iid() -> &'static ::intercom::IID {
                    & #iid_tokens
                }
            }
        ) );

        // If this is a trait (as opposed to an implicit struct `impl`), include
        // the Rust-to-COM call implementations.
        //
        // If the [com_interface] is on an implicit struct `impl` we'd end up with
        // `impl StructName for intercom::ComItf<StructName>`, which is invalid
        // syntax when `StructName` is struct instead of a trait.
        if itf.item_type() == utils::InterfaceType::Trait {

            // Gather method implementations.
            let impls = itf_variant.methods().iter()
                    .map( |m| rust_to_com_delegate( m, &vtable_ident ) );

            let unsafety = if itf.is_unsafe() { quote!( unsafe ) } else { quote!() };
            output.push( quote!(
                #unsafety impl #itf_ident for ::intercom::ComItf< #itf_ident > {
                    #( #impls )*
                }
            ) );
        }
    }
}

/// Creates the functions responsible for delegating calls from Rust to COM
/// interfaces.
///
/// # Arguments
///
/// * `method_info` - Method to delegate.
/// * `vtable_ident` - Vtable to use for the delegation.
fn rust_to_com_delegate(
    method_info : &ComMethodInfo,
    vtable_ident : &Ident,
) -> TokenStream {

    // Format the method arguments into tokens.
    let impl_args = method_info.args.iter().map( |ca| {
        let name = &ca.name;
        let ty = &ca.ty;
        quote!( #name : #ty )
    } );

    // The COM out-arguments that mirror the Rust return value will
    // require temporary variables during the COM call. Format their
    // declarations.
    let out_arg_declarations = method_info.returnhandler.com_out_args()
            .iter()
            .map( |ca| {
                let ident = &ca.name;
                let ty = &ca.handler.com_ty();
                let default = ca.handler.default_value();
                quote!( let mut #ident : #ty = #default; )
            } ).collect::<Vec<_>>();

    // Format the in and out parameters for the COM call.
    let ( temporaries, params ) : ( Vec<_>, Vec<_> ) = method_info.raw_com_args()
            .into_iter()
            .map( |com_arg| {
                let name = com_arg.name;
                match com_arg.dir {
                    Direction::In => {
                        let param = com_arg.handler.rust_to_com( &name );
                        ( param.temporary, param.value )
                    },
                    Direction::Out | Direction::Retval
                        => ( None, quote!( &mut #name ) ),
                }
            } )
            .unzip();

    // Combine the parameters into the final parameter list.
    // This includes the 'this' pointer and both the IN and OUT
    // parameters.
    let params = iter::once( quote!( comptr ) ).chain( params );

    // Create the return statement. 
    let return_ident = Ident::new( "__result", Span::call_site() );
    let return_statement = method_info
            .returnhandler
            .com_to_rust_return( &return_ident );

    // Resolve some of the fields needed for quote.
    let method_ident = &method_info.unique_name;
    let method_rust_ident = &method_info.display_name;
    let self_arg = &method_info.rust_self_arg;
    let return_ty = &method_info.rust_return_ty;
    let unsafety = if method_info.is_unsafe { quote!( unsafe ) } else { quote!() };

    // Construct the final method.
    quote!(
        #unsafety fn #method_rust_ident(
            #self_arg, #( #impl_args ),*
        ) -> #return_ty
        {
            #[allow(unused_imports)]
            use ::intercom::ComInto;

            let comptr = ::intercom::ComItf::ptr( self );
            let vtbl = comptr as *const *const #vtable_ident;

            #( #temporaries )*

            // Use an IIFE to act as a try/catch block. The various template
            // substitutions might end up using ?'s for error handling. The IIFE allows
            // us to handle the results here immediately.
            #[allow(unused_unsafe)]  // The fn itself _might_ be unsafe.
            let result : Result< #return_ty, ::intercom::ComError > = ( || unsafe {
                #( #out_arg_declarations )*;
                let #return_ident = ((**vtbl).#method_ident)( #( #params ),* );

                Ok( { #return_statement } )
            } )();

            #[allow(unused_imports)]
            use ::intercom::ErrorValue;
            match result {
                Ok( v ) => v,
                Err( err ) => < #return_ty as ErrorValue >::from_error(
                        ::intercom::return_hresult( err ) ),
            }
        }
    )
}
