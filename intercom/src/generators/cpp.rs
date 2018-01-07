
//! Enables the generation of header and source files for using intercom
//! libraries from C++ projects.

extern crate std;

use std::io::Write;
use std::path::Path;

use super::GeneratorError;

use intercom_common::foreign_ty::*;
use intercom_common::guid::*;
use intercom_common::methodinfo;
use intercom_common::model;
use intercom_common::utils;

use handlebars::Handlebars;

#[derive(PartialEq, Serialize, Debug)]
pub struct CppModel {
    lib_name : String,
    interfaces: Vec<CppInterface>,
    coclasses: Vec<CppCoClass>,
}

#[derive(PartialEq, Serialize, Debug)]
pub struct CppInterface {
    name : String,
    iid_struct : String,
    base : Option<String>,
    methods : Vec<CppMethod>,
}

#[derive(PartialEq, Serialize, Debug)]
struct CppMethod {
    pub name : String,
    pub ret_type : String,
    pub args : Vec<CppArg>,
}

#[derive(PartialEq, Serialize, Debug)]
struct CppArg {
    pub name : String,
    pub arg_type : String,
}

#[derive(PartialEq, Serialize, Debug)]
struct CppCoClass {
    pub name : String,
    pub clsid_struct : String,
    pub interface_count : usize,
    pub interfaces : Vec<String>,
}

impl CppModel {

    /// Converts the parse result into an header  that gets written to stdout.
    pub fn from_crate(
        c : &model::ComCrate,
    ) -> Result<CppModel, GeneratorError> {

        let foreign = CTyHandler;
        let lib = c.lib().as_ref().ok_or( GeneratorError::MissingLibrary )?;

        // Introduce all interfaces so we don't get errors on undeclared items.
        let interfaces = c.interfaces().iter().map( |(_, itf)| {

            // Get the method definitions for the current interface.
            let methods = itf.methods().iter().map( |m| {

                // Construct the argument list.
                let args = m.raw_com_args().iter().map( |a| {

                    // Argument direction affects both the argument attribute and
                    // whether the argument is passed by pointer or value.
                    let out_ptr = match a.dir {
                        methodinfo::Direction::In
                            => "",
                        methodinfo::Direction::Out 
                            | methodinfo::Direction::Retval
                            => "*",
                    };

                    // Get the foreign type for the arg type in C++ format.
                    let ty_name = to_cpp_type( c, &a.arg.ty )
                            .ok_or_else( || GeneratorError::UnsupportedType(
                                            utils::ty_to_string( &a.arg.ty ) ) )?;
                    Ok( CppArg {
                        name : a.arg.name.to_string(),
                        arg_type : format!( "{}{}", ty_name, out_ptr ),
                    } )

                } ).collect::<Result<Vec<_>, GeneratorError>>()?;

                let ret_ty = m.returnhandler.com_ty();
                let ret_ty_name = to_cpp_type( c, &ret_ty )
                        .ok_or_else( || GeneratorError::UnsupportedType(
                                        utils::ty_to_string( &ret_ty ) ) )?;
                Ok( CppMethod {
                    name: utils::pascal_case( m.name.as_ref() ),
                    ret_type: ret_ty_name,
                    args: args
                } )

            } ).collect::<Result<Vec<_>, GeneratorError>>()?;

            Ok( CppInterface {
                name: foreign.get_name( c, itf.name() ),
                iid_struct: guid_as_struct( itf.iid() ),
                base: itf.base_interface().as_ref().map( |i| foreign.get_name( c, i ) ),
                methods : methods,
            } )

        } ).collect::<Result<Vec<_>, GeneratorError>>()?;

        // Generate class descriptors.
        let classes = lib.coclasses().iter().map( |class_name| {

            // Get the class details by matching the name.
            let coclass  = &c.structs()[ class_name.as_ref() ];

            // Create a list of interfaces to be declared in the class descriptor.
            let interfaces = coclass.interfaces().iter().map(|itf_name| {
                foreign.get_name( c, itf_name )
            } ).collect();

            let clsid = coclass.clsid().as_ref()
                    .ok_or_else( || GeneratorError::MissingClsid(
                                        coclass.name().to_string() ) )?;

            Ok( CppCoClass {
                name : class_name.to_string(),
                clsid_struct : guid_as_struct( clsid ),
                interface_count : coclass.interfaces().len(),
                interfaces : interfaces,
            } )

        } ).collect::<Result<Vec<_>, GeneratorError>>()?;

        Ok( CppModel {
            lib_name : lib.name().to_owned(),
            interfaces : interfaces,
            coclasses : classes,
        } )
    }
}

pub fn write(
    path : &Path,
    out_header : Option< &mut Write >,
    out_source : Option< &mut Write >,
) -> Result<(), GeneratorError>
{
    // Parse the sources and convert the result into an IDL.
    let krate = if path.is_file() {
            model::ComCrate::parse_cargo_toml( path )
        } else {
            model::ComCrate::parse_cargo_toml( &path.join( "Cargo.toml" ) )
        }.map_err( |e| GeneratorError::CrateParseError( e ) )?;

    let model = CppModel::from_crate( &krate )?;

    if let Some( out ) = out_header {

        let mut reg = Handlebars::new();
        reg.register_template_string(
                "cpp_header",
                include_str!( "cpp_header.hbs" ) )
            .expect( "Error in the built-in C++ header template." );

        let rendered = reg
                .render( "cpp_header", &model )
                .expect( "Rendering a valid ComCrate to C++ header failed" );
        write!( out, "{}", rendered )?;
    }

    if let Some( out ) = out_source {

        let mut reg = Handlebars::new();
        reg.register_template_string(
                "cpp_source",
                include_str!( "cpp_source.hbs" ) )
            .expect( "Error in the built-in C++ source template." );

        let rendered = reg
                .render( "cpp_source", &model )
                .expect( "Rendering a valid ComCrate to C++ source failed" );
        write!( out, "{}", rendered )?;
    }

    Ok(())
}

/// Converts a guid to binarys representation.
pub fn guid_as_struct(
    g: &GUID
) -> String {

    format!( "{{0x{:08x},0x{:04x},0x{:04x},{{0x{:02x},0x{:02x},0x{:02x},0x{:02x},0x{:02x},0x{:02x},0x{:02x},0x{:02x}}}}}",
            g.data1, g.data2, g.data3,
            g.data4[0], g.data4[1], g.data4[2], g.data4[3],
            g.data4[4], g.data4[5], g.data4[6], g.data4[7] )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn crate_to_cpp() {

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

        let expected_cpp = CppModel {
            lib_name : "com_library".to_owned(),
            interfaces : vec![
                CppInterface {
                    name : "IInterface".to_owned(),
                    base : None,
                    iid_struct : "{0x22223333,0x4444,0x5555,{0x66,0x66,0x77,0x77,0x88,0x88,0x99,0x99}}".to_owned(),
                    methods : vec![
                        CppMethod {
                            name : "Method".to_owned(),
                            ret_type : "intercom::HRESULT".to_owned(),
                            args : vec![
                                CppArg { 
                                    name : "a".to_owned(),
                                    arg_type : "uint32_t".to_owned(),
                                },
                                CppArg { 
                                    name : "__out".to_owned(),
                                    arg_type : "bool*".to_owned(),
                                },
                            ]
                        }
                    ]
                },
                CppInterface {
                    name : "ICoClass".to_owned(),
                    base : Some( "IUnknown".to_owned() ),
                    iid_struct : "{0x44445555,0x6666,0x7777,{0x88,0x88,0x99,0x99,0x00,0x00,0xaa,0xaa}}".to_owned(),
                    methods : vec![
                        CppMethod {
                            name : "ComMethod".to_owned(),
                            ret_type : "void".to_owned(),
                            args : vec![
                                CppArg { 
                                    name : "b".to_owned(),
                                    arg_type : "uint32_t".to_owned(),
                                },
                            ]
                        }
                    ]
                }
            ],
            coclasses : vec![
                CppCoClass {
                    name : "CoClass".to_owned(),
                    clsid_struct : "{0x33334444,0x5555,0x6666,{0x77,0x77,0x88,0x88,0x99,0x99,0x00,0x00}}".to_owned(),
                    interface_count: 2,
                    interfaces : vec![
                        "ICoClass".to_owned(),
                        "IInterface".to_owned(),
                    ],
                }
            ],
        };

        let actual_cpp = CppModel::from_crate( &krate ).unwrap();

        assert_eq!( expected_cpp, actual_cpp );
    }
}
