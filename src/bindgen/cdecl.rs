/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;

use bindgen::config::Language;
use bindgen::declarationtyperesolver::DeclarationType;
use bindgen::ir::{Function, Type};
use bindgen::writer::{ListType, SourceWriter};

// This code is for translating Rust types into C declarations.
// See Section 6.7, Declarations, in the C standard for background.
// http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf

enum CDeclarator {
    Ptr(bool),
    Ref,
    Array(String),
    Func(Vec<(Option<String>, CDecl)>, bool),
}

impl CDeclarator {
    fn is_ptr(&self) -> bool {
        match self {
            &CDeclarator::Ptr(..) => true,
            &CDeclarator::Ref => true,
            &CDeclarator::Func(..) => true,
            _ => false,
        }
    }
}

struct CDecl {
    type_qualifers: String,
    type_name: String,
    type_generic_args: Vec<Type>,
    declarators: Vec<CDeclarator>,
    type_ctype: Option<DeclarationType>,
}

impl CDecl {
    fn new() -> CDecl {
        CDecl {
            type_qualifers: String::new(),
            type_name: String::new(),
            type_generic_args: Vec::new(),
            declarators: Vec::new(),
            type_ctype: None,
        }
    }

    fn from_type(t: &Type, lang: Language) -> CDecl {
        let mut cdecl = CDecl::new();
        cdecl.build_type(t, false, lang);
        cdecl
    }
    fn from_func(f: &Function, layout_vertical: bool, lang: Language) -> CDecl {
        let mut cdecl = CDecl::new();
        cdecl.build_func(f, layout_vertical, lang);
        cdecl
    }

    fn build_func(&mut self, f: &Function, layout_vertical: bool, lang: Language) {
        let args = f
            .args
            .iter()
            .map(|&(ref arg_name, ref arg_ty)| (Some(arg_name.clone()), CDecl::from_type(arg_ty, lang)))
            .collect();
        self.declarators
            .push(CDeclarator::Func(args, layout_vertical));
        self.build_type(&f.ret, false, lang);
    }

    fn build_type(&mut self, t: &Type, is_const: bool, lang: Language) {
        match t {
            &Type::Path(ref generic) => {
                if is_const {
                    assert!(
                        self.type_qualifers.len() == 0,
                        "error generating cdecl for {:?}",
                        t
                    );
                    self.type_qualifers = "const".to_owned();
                }

                assert!(
                    self.type_name.len() == 0,
                    "error generating cdecl for {:?}",
                    t
                );
                self.type_name = generic.export_name().to_owned();
                assert!(
                    self.type_generic_args.len() == 0,
                    "error generating cdecl for {:?}",
                    t
                );
                self.type_generic_args = generic.generics().to_owned();
                self.type_ctype = generic.ctype().cloned();
            }
            &Type::Primitive(ref p) => {
                if is_const {
                    assert!(
                        self.type_qualifers.len() == 0,
                        "error generating cdecl for {:?}",
                        t
                    );
                    self.type_qualifers = "const".to_owned();
                }

                assert!(
                    self.type_name.len() == 0,
                    "error generating cdecl for {:?}",
                    t
                );
                self.type_name = p.to_repr(lang).to_owned();
            }

            &Type::ConstPtr(ref t) => {
                self.declarators.push(CDeclarator::Ptr(is_const));
                self.build_type(t, true, lang);
            }
            &Type::Ptr(ref t) => {
                self.declarators.push(CDeclarator::Ptr(is_const));
                self.build_type(t, false, lang);
            }
            &Type::Ref(ref t) => {
                self.declarators.push(CDeclarator::Ref);
                self.build_type(t, true, lang);
            }
            &Type::MutRef(ref t) => {
                self.declarators.push(CDeclarator::Ref);
                self.build_type(t, false, lang);
            }
            &Type::Array(ref t, ref constant) => {
                let len = constant.as_str().to_owned();
                self.declarators.push(CDeclarator::Array(len));
                self.build_type(t, is_const, lang);
            }
            &Type::FuncPtr(ref ret, ref args) => {
                let args = args
                    .iter()
                    .map(|(ref name, ref ty)| (name.clone(), CDecl::from_type(ty, lang)))
                    .collect();
                self.declarators.push(CDeclarator::Ptr(false));
                self.declarators.push(CDeclarator::Func(args, false));
                self.build_type(ret, false, lang);
            }
        }
    }

    fn write<F: Write>(
        &self,
        out: &mut SourceWriter<F>,
        ident: Option<&str>,
        void_prototype: bool,
        lang: Language,
    ) {
        let cs_array_sz = if lang == Language::CS && self.declarators.len() == 1 {
            if let CDeclarator::Array(ref constant) = self.declarators[0] {
                Some(constant)
            }
            else { None }
        } else { None };

        if let Some(x) = cs_array_sz {
            write!(out, "[MarshalAs(UnmanagedType.ByValArray, SizeConst={})] readonly ", x);
        }

        // Write the type-specifier and type-qualifier first
        if self.type_qualifers.len() != 0 {
            write!(out, "{} ", self.type_qualifers);
        }

        if let Some(ref ctype) = self.type_ctype {
            write!(out, "{} ", ctype.to_str());
        }

        if let Some(_) = cs_array_sz {
            write!(out, "{}[]", self.type_name);
        }
        else {
            write!(out, "{}", self.type_name);
        }

        if !self.type_generic_args.is_empty() {
            out.write("<");
            out.write_horizontal_source_list(&self.type_generic_args, ListType::Join(", "));
            out.write(">");
        }

        // When we have an identifier, put a space between the type and the declarators
        if ident.is_some() {
            out.write(" ");
        }

        // Write the left part of declarators before the identifier
        let mut iter_rev = self.declarators.iter().rev().peekable();

        while let Some(declarator) = iter_rev.next() {
            let next_is_pointer = iter_rev.peek().map_or(false, |x| x.is_ptr());

            match declarator {
                &CDeclarator::Ptr(ref is_const) => {
                    if *is_const {
                        out.write("*const ");
                    } else {
                        out.write("*");
                    }
                }
                &CDeclarator::Ref => {
                    out.write("&");
                }
                &CDeclarator::Array(..) => {
                    if next_is_pointer {
                        out.write("(");
                    }
                }
                &CDeclarator::Func(..) => {
                    if next_is_pointer {
                        out.write("(");
                    }
                }
            }
        }

        // Write the identifier
        if let Some(ident) = ident {
            write!(out, "{}", ident);
        }

        // Write the right part of declarators after the identifier
        let mut iter = self.declarators.iter();
        let mut last_was_pointer = false;

        while let Some(declarator) = iter.next() {
            match declarator {
                &CDeclarator::Ptr(..) => {
                    last_was_pointer = true;
                }
                &CDeclarator::Ref => {
                    last_was_pointer = true;
                }
                &CDeclarator::Array(ref constant) => {
                    if last_was_pointer {
                        out.write(")");
                    }
                    if lang != Language::CS {
                        write!(out, "[{}]", constant);
                    }

                    last_was_pointer = false;
                }
                &CDeclarator::Func(ref args, layout_vertical) => {
                    if last_was_pointer {
                        out.write(")");
                    }

                    out.write("(");
                    if args.is_empty() && void_prototype {
                        out.write("void");
                    }
                    if layout_vertical {
                        let align_length = out.line_length_for_align();
                        out.push_set_spaces(align_length);
                        for (i, &(ref arg_ident, ref arg_ty)) in args.iter().enumerate() {
                            if i != 0 {
                                out.write(",");
                                out.new_line();
                            }

                            // Convert &Option<String> to Option<&str>
                            let arg_ident = arg_ident.as_ref().map(|x| x.as_ref());

                            arg_ty.write(out, arg_ident, void_prototype, lang);
                        }
                        out.pop_tab();
                    } else {
                        for (i, &(ref arg_ident, ref arg_ty)) in args.iter().enumerate() {
                            if i != 0 {
                                out.write(", ");
                            }

                            // Convert &Option<String> to Option<&str>
                            let arg_ident = arg_ident.as_ref().map(|x| x.as_ref());

                            arg_ty.write(out, arg_ident, void_prototype, lang);
                        }
                    }
                    out.write(")");

                    last_was_pointer = true;
                }
            }
        }
    }
}

pub fn write_func<F: Write>(
    out: &mut SourceWriter<F>,
    f: &Function,
    layout_vertical: bool,
    void_prototype: bool,
    lang: Language,
) {
    &CDecl::from_func(f, layout_vertical, lang).write(out, Some(f.path().name()), void_prototype, lang);
}

pub fn write_field<F: Write>(out: &mut SourceWriter<F>, t: &Type, ident: &str, lang: Language) {
    &CDecl::from_type(t, lang).write(out, Some(ident), false, lang);
}

pub fn write_type<F: Write>(out: &mut SourceWriter<F>, t: &Type, lang: Language) {
    &CDecl::from_type(t, lang).write(out, None, false, lang);
}
