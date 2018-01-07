use std::io::Write;
use std::path::Path;

use super::GeneratorError;

use intercom_common::utils;
use intercom_common::model;
use intercom_common::methodinfo;
use intercom_common::foreign_ty::*;

use handlebars::Handlebars;

#[derive(PartialEq, Serialize, Debug)]
struct IdlModel {
    pub lib_id : String,
    pub lib_name : String,
    pub interfaces : Vec<IdlInterface>,
    pub coclasses : Vec<IdlCoClass>,
}

#[derive(PartialEq, Serialize, Debug)]
struct IdlInterface {
    pub name : String,
    pub base : Option<String>,
    pub iid : String,
    pub methods : Vec<IdlMethod>,
}

#[derive(PartialEq, Serialize, Debug)]
struct IdlMethod {
    pub name : String,
    pub idx : usize,
    pub ret_type : String,
    pub args : Vec<IdlArg>,
}

#[derive(PartialEq, Serialize, Debug)]
struct IdlArg {
    pub name : String,
    pub arg_type : String,
    pub attributes : String,
}

#[derive(PartialEq, Serialize, Debug)]
struct IdlCoClass {
    pub name : String,
    pub clsid : String,
    pub interfaces : Vec<String>,
}

impl IdlModel {

    /// Converts the parse result into an IDL that gets written to stdout.
    fn from_crate(
        c : &model::ComCrate
    ) -> Result<IdlModel, GeneratorError> {

        let foreign = CTyHandler;
        let lib = c.lib().as_ref().ok_or( GeneratorError::MissingLibrary )?;

        // Define all interfaces.
        let itfs = c.interfaces().iter().map(|(_, itf)| {

            // Get the method definitions for the current interface.
            let methods = itf.methods().iter().enumerate().map(|(i,m)| {

                // Construct the argument list.
                let args = m.raw_com_args().iter().map(|a| {

                    // Argument direction affects both the argument attribute and
                    // whether the argument is passed by pointer or value.
                    let ( attrs, out_ptr ) = match a.dir {
                        methodinfo::Direction::In => ( "in", "" ),
                        methodinfo::Direction::Out => ( "out", "*" ),
                        methodinfo::Direction::Retval => ( "out, retval", "*" ),
                    };

                    // Get the foreign type for the arg type.
                    let ty_name = foreign
                        .get_ty( c, &a.arg.ty )
                        .ok_or_else( || GeneratorError::UnsupportedType(
                                        utils::ty_to_string( &a.arg.ty ) ) )?;

                    Ok( IdlArg {
                        name : a.arg.name.to_string(),
                        arg_type : format!( "{}{}", ty_name, out_ptr ),
                        attributes : attrs.to_owned(),
                    } )

                } ).collect::<Result<Vec<_>, GeneratorError>>()?;

                let ret_ty = m.returnhandler.com_ty();
                let ret_ty_name = foreign
                        .get_ty( c, &ret_ty )
                        .ok_or_else( || GeneratorError::UnsupportedType(
                                        utils::ty_to_string( &ret_ty ) ) )?;
                Ok( IdlMethod {
                    name: utils::pascal_case( m.name.as_ref() ),
                    idx: i,
                    ret_type: ret_ty_name,
                    args: args
                } )

            } ).collect::<Result<Vec<_>, GeneratorError>>()?;

            // Now that we have methods sorted out, we can construct the final
            // interface definition.
            Ok( IdlInterface {
                name: foreign.get_name( c, itf.name() ),
                base: itf.base_interface().as_ref().map( |i| foreign.get_name( c, i ) ),
                iid: format!( "{:-X}", itf.iid() ),
                methods: methods,
            } )

        } ).collect::<Result<Vec<_>, GeneratorError>>()?;

        // Create coclass definitions.
        //
        // Here r.class_names contains the class names that were defined in the
        // [com_library] attribute. This is our source for the classes to include
        // in the IDL. r.classes has the actual class details, but might include
        // classes that are not exposed by the library.
        let classes = lib.coclasses().iter().map(|class_name| {

            // Get the class details by matching the name.
            let coclass = &c.structs()[ class_name.as_ref() ];

            // Get the interfaces the class implements.
            let interfaces = coclass.interfaces().iter().map(|itf_name| {
                foreign.get_name( c, itf_name )
            } ).collect();

            let clsid = coclass.clsid().as_ref()
                    .ok_or_else( || GeneratorError::MissingClsid(
                                        coclass.name().to_string() ) )?;
            Ok( IdlCoClass {
                name : coclass.name().to_string(),
                clsid: format!( "{:-X}", clsid ),
                interfaces: interfaces
            } )
        } ).collect::<Result<_,GeneratorError>>()?;

        Ok( IdlModel {
            lib_id : format!( "{:-X}", lib.libid() ),
            lib_name : utils::pascal_case( lib.name() ),
            interfaces : itfs,
            coclasses : classes,
        } )
    }
}

#[allow(dead_code)]
pub fn write( path : &Path, out : &mut Write ) -> Result<(), GeneratorError> {

    // Parse the sources and convert the result into an IDL.
    let krate = if path.is_file() {
            model::ComCrate::parse_cargo_toml( path )
        } else {
            model::ComCrate::parse_cargo_toml( &path.join( "Cargo.toml" ) )
        }.map_err( |e| GeneratorError::CrateParseError( e ) )?;

    let model = IdlModel::from_crate( &krate )?;
    let mut reg = Handlebars::new();
    reg.register_template_string( "idl", include_str!( "idl.hbs" ) )
            .expect( "Error in the built-in IDL template." );

    let rendered = reg
            .render( "idl", &model )
            .expect( "Rendering a valid ComCrate to IDL failed" );
    write!( out, "{}", rendered )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn crate_to_idl() {

        let krate = model::ComCrate::parse( "com_library", &[ r#"
            #[com_library( "11112222-3333-4444-5555-666677778888", CoClass )]
            struct S;

            #[com_interface( "22223333-4444-5555-6666-777788889999", NO_BASE )]
            trait IInterface {
                fn method( &self, a : u32 ) -> ComResult<bool>;
            }

            #[com_class( "33334444-5555-6666-7777-888899990000", CoClass, IInterface )]
            struct CoClass;

            #[com_interface( "44445555-6666-7777-8888-99990000AAAA" )]
            #[com_impl]
            impl CoClass {
                pub fn new() -> CoClass { CoClass }
                pub fn com_method( &self, b : u32 ) {}
            }

            #[com_impl]
            impl IInterface for CoClass {
                fn method( &self, a : u32 ) -> ComResult<bool> { unreachable!() }
            }
        "# ] ).expect( "Could not parse test crate" );

        let expected_idl = IdlModel {
            lib_id : "11112222-3333-4444-5555-666677778888".to_owned(),
            lib_name : "ComLibrary".to_owned(),
            interfaces : vec![
                IdlInterface {
                    name : "IInterface".to_owned(),
                    base : None,
                    iid : "22223333-4444-5555-6666-777788889999".to_owned(),
                    methods : vec![
                        IdlMethod {
                            name : "Method".to_owned(),
                            idx : 0,
                            ret_type : "HRESULT".to_owned(),
                            args : vec![
                                IdlArg { 
                                    name : "a".to_owned(),
                                    arg_type : "uint32".to_owned(),
                                    attributes : "in".to_owned(),
                                },
                                IdlArg { 
                                    name : "__out".to_owned(),
                                    arg_type : "bool*".to_owned(),
                                    attributes : "out, retval".to_owned(),
                                },
                            ]
                        }
                    ]
                },
                IdlInterface {
                    name : "ICoClass".to_owned(),
                    base : Some( "IUnknown".to_owned() ),
                    iid : "44445555-6666-7777-8888-99990000AAAA".to_owned(),
                    methods : vec![
                        IdlMethod {
                            name : "ComMethod".to_owned(),
                            idx : 0,
                            ret_type : "void".to_owned(),
                            args : vec![
                                IdlArg { 
                                    name : "b".to_owned(),
                                    arg_type : "uint32".to_owned(),
                                    attributes : "in".to_owned(),
                                },
                            ]
                        }
                    ]
                }
            ],
            coclasses : vec![
                IdlCoClass {
                    name : "CoClass".to_owned(),
                    clsid : "33334444-5555-6666-7777-888899990000".to_owned(),
                    interfaces : vec![
                        "ICoClass".to_owned(),
                        "IInterface".to_owned(),
                    ],
                }
            ],
        };

        let actual_idl = IdlModel::from_crate( &krate ).unwrap();

        assert_eq!( expected_idl, actual_idl );
    }
}