
use ::prelude::*;
use std::rc::Rc;
use syn::*;

use ast_converters::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction { In, Out, Retval }

pub struct TypeConversion {

    /// Possible temporary values that need to be kept alive for the duration
    /// of the conversion result usage.
    pub temporary: Option<TokenStream>,

    /// Conversion result value. Possibly referencing the temporary value.
    pub value : TokenStream,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct ModelTypeSystemConfig {
    pub effective_system : ModelTypeSystem,
    pub is_default : bool,
}

impl ModelTypeSystemConfig {
    pub fn get_unique_name( &self, base : &str ) -> String {
        match self.is_default {
            true => base.to_string(),
            false => format!( "{}_{:?}", base, self.effective_system ),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum ModelTypeSystem {

    /// COM Automation compatible type system.
    Automation,

    /// Raw type system.
    Raw,
}

impl ModelTypeSystem {

    /// Converts the model type system into public type system tokens.
    pub fn as_typesystem_tokens( self ) -> TokenStream {
        match self {
            ModelTypeSystem::Automation =>
                    quote!( ::intercom::TypeSystem::Automation ),
            ModelTypeSystem::Raw =>
                    quote!( ::intercom::TypeSystem::Raw ),
        }
    }
}

/// Type usage context.
pub struct TypeContext {
    type_system: ModelTypeSystem,
}

impl TypeContext {
    pub fn new( type_system : ModelTypeSystem ) -> TypeContext {
        TypeContext { type_system }
    }
}

/// Defines Type-specific logic for handling the various parameter types in the
/// Rust/COM interface.
pub trait TypeHandler {

    /// The Rust type.
    fn rust_ty( &self ) -> Type;

    /// The COM type.
    fn com_ty( &self, _dir: Direction ) -> Type
    {
        self.rust_ty()
    }

    /// Converts a COM parameter named by the ident into a Rust type.
    fn com_to_rust(
        &self,
        ident : &Ident,
        _dir: Direction,
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.into() ),
        }
    }

    /// Converts a Rust parameter named by the ident into a COM type.
    fn rust_to_com(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.into() )
        }
    }

    /// Gets the default value for the type.
    fn default_value( &self ) -> TokenStream
    {
        match self.rust_ty() {
            Type::Path( ref p ) => {
                let ident = p.path.get_ident().unwrap();
                let name = ident.to_string();
                match name.as_ref() {
                    "RawComPtr" => quote!( ::std::ptr::null_mut() ),
                    _ => quote!( Default::default() )
                }
            },
            Type::Ptr( .. ) => quote!( ::std::ptr::null_mut() ),
            _ => quote!( Default::default() )
        }
    }
}

/// Identity parameter handler.
///
/// No special logic.
struct IdentityParam( Type );

impl TypeHandler for IdentityParam {
    fn rust_ty( &self ) -> Type { self.0.clone() }
}


/// `ComItf` parameter handler. Supports `ComItf` Rust type and ensures the this
/// to/from `RawComPtr` COM type.
struct ComItfParam { ty: Type, context: TypeContext }

impl TypeHandler for ComItfParam {

    fn rust_ty( &self ) -> Type { self.ty.clone() }

    /// The COM type.
    fn com_ty( &self, _dir: Direction ) -> Type
    {
        let rust_ty = self.rust_ty();

        // Extract the interface type T from the ComItf<T> type definition
        use syn;
        let itf_ty = match rust_ty {
            syn::Type::Path( path ) =>
                match path.path.segments.last().unwrap().value().arguments {
                    syn::PathArguments::AngleBracketed( ref ab ) =>
                            match ab.args.last().unwrap().value() {
                                syn::GenericArgument::Type( ref t ) => t.clone(),
                                _ => panic!( "ComItf generic argument must be type" ),
                            },
                    _ => panic!( "ComItf type parameter must be angle bracketed" ),
                },
            _ => panic!( "ComItf type parameter must be a type path" ),
        };

        // Construct the final InterfacePtr<T> type.
        parse_quote!( ::intercom::raw::InterfacePtr< #itf_ty > )
    }

    fn default_value( &self ) -> TokenStream
    {
        quote!( ::intercom::raw::InterfacePtr::new( ::std::ptr::null_mut() ) )
    }

    /// Converts a COM parameter named by the ident into a Rust type.
    fn com_to_rust(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        let ts = self.context.type_system.as_typesystem_tokens();
        TypeConversion {
            temporary: None,
            value: quote!( ::intercom::ComItf::wrap( #ident, #ts ) ),
        }
    }

    /// Converts a Rust parameter named by the ident into a COM type.
    fn rust_to_com(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        let ts = self.context.type_system.as_typesystem_tokens();
        TypeConversion {
            temporary: None,
            value: quote!( ::intercom::ComItf::ptr( &#ident.into(), #ts ) )
        }
    }
}

/// `bool` parameter handler. Supports `VARIANT_BOOL` for automation type system.
struct BoolParam { context: TypeContext }

impl TypeHandler for BoolParam {

    fn rust_ty( &self ) -> Type { parse_quote!( bool ) }

    /// The COM type.
    fn com_ty( &self, _dir: Direction ) -> Type
    {
        match self.context.type_system {
            ModelTypeSystem::Automation => parse_quote!( ::intercom::raw::VariantBool ),
            ModelTypeSystem::Raw => parse_quote!( bool ),
        }
    }

    fn default_value( &self ) -> TokenStream
    {
        match self.context.type_system {
            ModelTypeSystem::Automation => quote!( false.into() ),
            ModelTypeSystem::Raw => quote!( false ),
        }
    }

    /// Converts a COM parameter named by the ident into a Rust type.
    fn com_to_rust(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.into() )
        }
    }

    /// Converts a Rust parameter named by the ident into a COM type.
    fn rust_to_com(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.into() )
        }
    }
}

/// `Variant` parameter handler. Supports `VARIANT` for automation type system.
struct VariantParam { ty: Type }

impl TypeHandler for VariantParam {

    fn rust_ty( &self ) -> Type { self.ty.clone() }

    /// The COM type.
    fn com_ty( &self, _dir: Direction ) -> Type
    {
        parse_quote!( ::intercom::raw::Variant )
    }

    /// Converts a COM parameter named by the ident into a Rust type.
    fn com_to_rust(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.into() )
        }
    }

    /// Converts a Rust parameter named by the ident into a COM type.
    fn rust_to_com(
        &self, ident : &Ident, _dir: Direction
    ) -> TypeConversion
    {
        TypeConversion {
            temporary: None,
            value: quote!( #ident.com_into()? )
        }
    }
}

/// String parameter handler. Converts between Rust String and COM BSTR types.
struct StringParam { ty: Type, context: TypeContext }
impl TypeHandler for StringParam
{
    fn rust_ty( &self ) -> Type { self.ty.clone() }

    fn com_ty( &self, dir: Direction ) -> Type
    {
        match self.context.type_system {
            ModelTypeSystem::Automation => match dir {
                Direction::In => parse_quote!( ::intercom::raw::InBSTR ),
                Direction::Out | Direction::Retval => parse_quote!( ::intercom::raw::OutBSTR ),
            },
            ModelTypeSystem::Raw => match dir {
                Direction::In => parse_quote!( ::intercom::raw::InCStr ),
                Direction::Out | Direction::Retval => parse_quote!( ::intercom::raw::OutCStr ),
            },
        }
    }

    fn com_to_rust( &self, ident : &Ident, dir: Direction ) -> TypeConversion
    {
        match dir {

            Direction::In => {

                let str_type = match self.context.type_system {
                    ModelTypeSystem::Automation => quote!( BStr ),
                    ModelTypeSystem::Raw => quote!( CStr ),
                };

                let target_ty = self.rust_ty();
                let intermediate_ty = quote!( &::intercom::#str_type );
                let to_intermediate = quote!( ::intercom::#str_type::from_ptr( #ident ) );
                let as_trait = quote!( < #target_ty as ::intercom::FromWithTemporary< #intermediate_ty > > );

                let temp_ident = Ident::new( &format!( "__{}_temporary", ident.to_string() ), Span::call_site() );
                TypeConversion {
                    temporary: Some( quote!( let mut #temp_ident = #as_trait::to_temporary( #to_intermediate )?; ) ),
                    value: quote!( #as_trait::from_temporary( &mut #temp_ident )? ),
                }
            },
            Direction::Out | Direction::Retval => {

                // The type system input 'ident' should represent either a BString or CString
                // depending on the type system.
                let str_type = match self.context.type_system {
                    ModelTypeSystem::Automation => quote!( BString::from_ptr ),
                    ModelTypeSystem::Raw => quote!( CString::from_raw ),
                };

                // Get the type system string as Rust string.
                let ts_string = quote!( ::intercom::#str_type( #ident ) );

                // Convert the TS string into whatever string type the method requires.
                TypeConversion {
                    temporary: None,
                    value: quote!( #ts_string.com_into()? ),
                }
            },
        }
    }

    fn rust_to_com( &self, ident : &Ident, dir: Direction ) -> TypeConversion
    {
        match dir {

            Direction::In => {

                let str_type = match self.context.type_system {
                    ModelTypeSystem::Automation => quote!( BStr ),
                    ModelTypeSystem::Raw => quote!( CStr ),
                };

                let target_ty = self.rust_ty();
                let intermediate_ty = quote!( &::intercom::#str_type );
                let as_trait = quote!( < #intermediate_ty as ::intercom::FromWithTemporary< #target_ty > > );

                let temp_ident = Ident::new( &format!( "__{}_temporary", ident.to_string() ), Span::call_site() );
                TypeConversion {
                    temporary: Some( quote!( let mut #temp_ident = #as_trait::to_temporary( #ident )?; ) ),
                    value: quote!( #as_trait::from_temporary( &mut #temp_ident )?.as_ptr() ),
                }
            },
            Direction::Out | Direction::Retval => {

                // The Rust string value `ident` must be first converted into a type system
                // compatible Rust string, either BString or CSTring.
                // depending on the type system.
                let str_type = match self.context.type_system {
                    ModelTypeSystem::Automation => quote!( BString ),
                    ModelTypeSystem::Raw => quote!( CString ),
                };

                // Convert the `ident` into the required string type.
                let ts_string = quote!( ::intercom::ComInto::< ::intercom::#str_type >::com_into( #ident )? );

                TypeConversion {
                    temporary: None,
                    value: match self.context.type_system {
                        ModelTypeSystem::Automation =>
                            quote!( #ts_string.into_ptr() ),
                        ModelTypeSystem::Raw =>
                            quote!( #ts_string.into_raw() ),
                    }
                }
            },
        }
    }

    fn default_value( &self ) -> TokenStream
    {
        quote!( ::std::ptr::null_mut() )
    }
}

/// Resolves the `TypeHandler` to use.
pub fn get_ty_handler(
    arg_ty : &Type,
    context : TypeContext,
) -> Rc<TypeHandler>
{
    let type_info = ::type_parser::parse( arg_ty )
            .unwrap_or_else( || panic!( "Type {:?} could not be parsed.", arg_ty ) );

    map_by_name(
            type_info.get_name().as_ref(), type_info.original.clone(),
            context )
}

/// Selects type handler based on the name of the type.
fn map_by_name(
    name: &str,
    original_type: Type,
    context: TypeContext,
) -> Rc<TypeHandler> {

    match name {

        "ComItf" => Rc::new( ComItfParam { ty: original_type, context } ),
        "CString" | "CStr" | "BString" | "BStr" | "String" | "str" =>
            Rc::new( StringParam { ty: original_type, context } ),
        "bool" =>
            Rc::new( BoolParam { context } ),
        "Variant" =>
            Rc::new( VariantParam { ty: original_type } ),
        // "str" => Rc::new( StringRefParam( original_type ) ),

        // Unknown. Use IdentityParam.
        _ => Rc::new( IdentityParam( original_type ) )
    }

}
