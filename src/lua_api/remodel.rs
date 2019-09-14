use std::{
    ffi::OsStr,
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::Path,
    sync::{Arc, Mutex},
};

use rlua::{UserData, UserDataMethods};

use super::LuaInstance;

pub struct Remodel;

impl UserData for Remodel {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("readPlaceFile", |_context, lua_path: String| {
            let path = Path::new(&lua_path);

            match path.extension().and_then(OsStr::to_str) {
                Some("rbxlx") => {
                    let file = BufReader::new(File::open(path).map_err(rlua::Error::external)?);
                    let tree = rbx_xml::from_reader_default(file).map_err(rlua::Error::external)?;

                    let root_id = tree.get_root_id();
                    let tree = Arc::new(Mutex::new(tree));

                    Ok(LuaInstance::new(tree, root_id))
                }
                Some("rbxl") => Err(rlua::Error::external(
                    "Reading rbxl place files is not supported yet.",
                )),
                _ => Err(rlua::Error::external(format!(
                    "Invalid place file path {}",
                    lua_path
                ))),
            }
        });

        methods.add_function("readModelFile", |_context, lua_path: String| {
            let path = Path::new(&lua_path);

            match path.extension().and_then(OsStr::to_str) {
                Some("rbxmx") => {
                    let file = BufReader::new(File::open(path).map_err(rlua::Error::external)?);
                    let tree = rbx_xml::from_reader_default(file).map_err(rlua::Error::external)?;

                    let tree = Arc::new(Mutex::new(tree));

                    let instances = {
                        let tree_handle = tree.lock().unwrap();

                        let root_id = tree_handle.get_root_id();
                        let root_instance = tree_handle.get_instance(root_id).unwrap();

                        root_instance
                            .get_children_ids()
                            .into_iter()
                            .copied()
                            .map(|id| LuaInstance::new(Arc::clone(&tree), id))
                            .collect::<Vec<_>>()
                    };

                    Ok(instances)
                }
                Some("rbxm") => Err(rlua::Error::external(
                    "Reading rbxm models files is not supported yet.",
                )),
                _ => Err(rlua::Error::external(format!(
                    "Invalid model file path {}",
                    lua_path
                ))),
            }
        });

        methods.add_function(
            "writePlaceFile",
            |_context, (lua_instance, lua_path): (LuaInstance, String)| {
                let path = Path::new(&lua_path);

                match path.extension().and_then(OsStr::to_str) {
                    Some("rbxlx") => {
                        let file =
                            BufWriter::new(File::create(&path).map_err(rlua::Error::external)?);

                        let tree = lua_instance.tree.lock().unwrap();
                        let instance = tree
                            .get_instance(lua_instance.id)
                            .ok_or_else(|| rlua::Error::external("Instance was destroyed"))?;

                        rbx_xml::to_writer_default(file, &tree, instance.get_children_ids())
                            .map_err(rlua::Error::external)?;

                        Ok(())
                    }
                    Some("rbxl") => Err(rlua::Error::external(
                        "Writing rbxl place files is not supported yet.",
                    )),
                    _ => Err(rlua::Error::external(format!(
                        "Invalid place file path {}",
                        lua_path
                    ))),
                }
            },
        );

        methods.add_function(
            "writeModelFile",
            |_context, (lua_instance, lua_path): (LuaInstance, String)| {
                let path = Path::new(&lua_path);

                match path.extension().and_then(OsStr::to_str) {
                    Some("rbxmx") => {
                        let file =
                            BufWriter::new(File::create(&path).map_err(rlua::Error::external)?);

                        let tree = lua_instance.tree.lock().unwrap();

                        rbx_xml::to_writer_default(file, &tree, &[lua_instance.id])
                            .map_err(rlua::Error::external)
                    }
                    Some("rbxm") => Err(rlua::Error::external(
                        "Writing rbxm model files is not supported yet.",
                    )),
                    _ => Err(rlua::Error::external(format!(
                        "Invalid model file path {}",
                        lua_path
                    ))),
                }
            },
        );

        methods.add_function("createDirAll", |_context, path: String| {
            create_dir_all(path).map_err(rlua::Error::external)
        });
    }
}