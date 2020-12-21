use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
    path::Path,
};

use super::Result;

const MAX_TUPLE_SIZE: u32 = 8;

pub fn generate_impls(out_dir: &Path) -> Result {
    let type_list_impls = File::create(out_dir.join("type_list.rs"))?;
    let fold_impls = File::create(out_dir.join("fold.rs"))?;
    let map_impls = File::create(out_dir.join("map.rs"))?;
    let zip_impls = File::create(out_dir.join("zip.rs"))?;

    build_type_list_impls(&type_list_impls)?;
    build_fold_impls(&fold_impls)?;
    build_map_impls(&map_impls)?;
    build_zip_impls(&zip_impls)?;

    Ok(())
}

struct Vars {
    name:    char,
    prefix:  &'static dyn Display,
    postfix: &'static dyn Display,
    count:   u32,
}

struct ZipVars {
    name:  (char, char),
    count: u32,
}

impl Display for Vars {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 1..=self.count {
            write!(f, "{}{}{}{}", self.prefix, self.name, i, self.postfix)?;
        }

        Ok(())
    }
}

impl Display for ZipVars {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 1..=self.count {
            write!(f, "({}{id},{}{id}),", self.name.0, self.name.1, id = i)?;
        }

        Ok(())
    }
}

fn build_type_list_impls(mut out: &File) -> Result {
    for i in 0..=MAX_TUPLE_SIZE {
        write!(
            out,
            "
impl<Parent, {impl_generics}> FieldList<Parent> for ({type_generics}) {{
    type Type = ({assoc_type});
    type TypeMut = ({assoc_type_mut});

    #[allow(unused_variables)]
    unsafe fn project_raw(&self, ptr: *const Parent) -> Self::Type {{
        let ({vars}) = self;
        ({project_raw})
    }}

    #[allow(unused_variables)]
    unsafe fn project_raw_mut(&self, ptr: *mut Parent) -> Self::TypeMut {{
        let ({vars}) = self;
        ({project_raw_mut})
    }}
}}
            ",
            impl_generics = Vars {
                prefix:  &"",
                postfix: &": Field<Parent = Parent>,",
                count:   i,
                name:    'A',
            },
            type_generics = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'A',
            },
            assoc_type = Vars {
                prefix:  &"*const ",
                postfix: &"::Type,",
                count:   i,
                name:    'A',
            },
            assoc_type_mut = Vars {
                prefix:  &"*mut ",
                postfix: &"::Type,",
                count:   i,
                name:    'A',
            },
            vars = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'a',
            },
            project_raw = Vars {
                prefix:  &"",
                postfix: &".project_raw(ptr),",
                count:   i,
                name:    'a',
            },
            project_raw_mut = Vars {
                prefix:  &"",
                postfix: &".project_raw_mut(ptr),",
                count:   i,
                name:    'a',
            },
        )?
    }
    Ok(())
}

fn build_fold_impls(mut out: &File) -> Result {
    for i in 0..=MAX_TUPLE_SIZE {
        write!(
            out,
            "
impl<A, F, {type_generics}> ListFold<A, F> for ({type_generics})
where crate::List!({type_generics}): ListFold<A, F> {{
    type Output = <crate::List!({type_generics}) as ListFold<A, F>>::Output;

    fn fold(self, acc: A, f: F) -> Self::Output {{
        let ({vars}) = self;
        crate::list!({vars}).fold(acc, f)
    }}
}}
            ",
            type_generics = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'A',
            },
            vars = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'a',
            },
        )?
    }
    Ok(())
}

fn build_map_impls(mut out: &File) -> Result {
    for i in 0..=MAX_TUPLE_SIZE {
        write!(
            out,
            "
impl<F, {type_generics}> ListMap<F> for ({type_generics})
where {where_clause} {{
    type Output = ({output});

    fn map(self, f: F) -> Self::Output {{
        let ({vars}) = self;
        let crate::list_pat!({vars}) = crate::list!({vars}).map(f);
        ({vars})
    }}
}}
            ",
            type_generics = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'A',
            },
            where_clause = Vars {
                prefix:  &"F: CallMut<(",
                postfix: &",)>,",
                count:   i,
                name:    'A',
            },
            vars = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'a',
            },
            output = Vars {
                prefix:  &"<F as CallOnce<(",
                postfix: &",)>>::Output,",
                count:   i,
                name:    'A',
            }
        )?
    }
    Ok(())
}

fn build_zip_impls(mut out: &File) -> Result {
    for i in 0..=MAX_TUPLE_SIZE {
        write!(
            out,
            "
        impl<{a_type_generics} {b_type_generics}> ListZip<({b_type_generics})> \
             for ({a_type_generics}) {{
    type Output = ({output});

    fn zip(self, ({b_vars}): ({b_type_generics})) -> Self::Output {{
        let ({a_vars}) = self;
        ({zipped})
    }}
}}
            ",
            a_type_generics = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'A',
            },
            b_type_generics = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'B',
            },
            a_vars = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'a',
            },
            b_vars = Vars {
                prefix:  &"",
                postfix: &",",
                count:   i,
                name:    'b',
            },
            output = ZipVars {
                name:  ('A', 'B'),
                count: i,
            },
            zipped = ZipVars {
                name:  ('a', 'b'),
                count: i,
            }
        )?
    }
    Ok(())
}
